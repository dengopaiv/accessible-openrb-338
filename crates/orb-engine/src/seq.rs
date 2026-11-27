//! The sequencer: the song position, the track events, the chase, the loop and the transport
//! (`docs/initial_investigation.md` section 5).
//!
//! The DLL's sequencer object sits at `song+8` (ctor `sub_1001EC50`): the position as a double at
//! `+0`, the last tick at `+8`, the song-mode tick saved by pattern mode at `+0x10`, the track
//! count at `+0xC` and the 32 track pointers at `+0x14` (nine are used, one per device; their
//! back pointer to the song is `+0x94`). The song's global struct (`song+0x158`,
//! [`song::Globals`]) holds the transport state the sequencer reads. [`Sequencer::tick`] is
//! `sub_1001EF20`, which the mixer block `sub_10013444` runs once per block before any device
//! renders (0x1001346A). Every device call made here therefore lands between two blocks: the
//! device sees it at sample 0 of the block that follows.
//!
//! A track (ctor `sub_1001EDA0`, 0x22C bytes) is a sorted circular list (list ctor `sub_100144B0`
//! at `track+0x108`: the head node is the list itself, with tick `0x7FFFFFFF`), a pointer to the
//! last applied node (`+0x228`), the chase state (`+0x84`, 128 bytes), a per-controller hold mask
//! (`+4`, 128 bytes) and the device id (`+0x104`). [`Track`]'s list is a slice of the song's
//! [`Pool`] in list order; `next` is the index of the node after the last applied one. A
//! second list at `track+0x198` holds what is being recorded (task-057,
//! [`Sequencer::record_hook`]) until the merge `sub_1001F520` moves it into the first.
//!
//! **The node pool.** The DLL takes every list node from one pool of [`POOL_NODES`] nodes
//! (`dword_10100230`, built once by `EngineOpen` → `sub_1001C7B0` → `sub_10021A10` →
//! `sub_100142C0`, freed by `sub_1001EC00`), shared by every list of the process: the song
//! ctor's inserts (`sub_1001F900`), msg 37 (`sub_10021010`), the record hook and the merge all
//! go through `sub_10014750`, and `sub_10014434` refuses a node when one or none is free: the
//! insert then stores nothing (task-082). orb gives each song one pool of that size, [`Pool`]:
//! one buffer of `POOL_NODES` nodes reserved when the song is built, holding the song's 18
//! lists back to back, so that recording, which inserts in the audio callback, never grows a
//! `Vec` there. A [`Track`] holds its lists' place in the pool, not the nodes.
//!
//! Where orb differs: orb-player keeps a `pending` song beside the `current` one while a loaded
//! song waits for Play, and each has its own pool, so two pools are live then; in the DLL the
//! one pool of the instance would serve both songs. The two agree unless the two songs together
//! hold more than [`POOL_NODES`] nodes (then the DLL refuses nodes orb still gives).

#[cfg(test)]
mod merge_tests;
#[cfg(test)]
mod tests;

use crate::frame::Devices;
use crate::song::{self, Device, Globals};

/// `0x186A0` at 0x1001431B: the nodes of the DLL's event pool.
pub const POOL_NODES: usize = 100_000;
/// The tracks of a song, one per device.
pub const TRACKS: usize = 9;
/// The lists of a song's [`Pool`]: list `k < 9` is track k's (`+0x108`), list `9 + k` its
/// recording (`+0x198`).
pub const LISTS: usize = 2 * TRACKS;

/// A song's node pool (the module doc): its [`LISTS`] lists, each in list order, back to back
/// in one buffer whose capacity of [`POOL_NODES`] nodes is reserved when the pool is made. An
/// insert or a removal moves the nodes after it within that capacity, so a pooled insert, which
/// takes a node only while more than one is free, never allocates. `pool[k]` is list k.
///
/// The tracks' lists come first, so a recorded node moves only the recording lists after it.
/// The merge ([`Track::merge`]) moves the nodes after a track's list a few times per track,
/// not once per node it inserts (task-084).
#[derive(Debug, PartialEq, Eq)]
pub struct Pool {
    /// The nodes of every list.
    nodes: Vec<Event>,
    /// The end of each list in `nodes`; list k starts where list k-1 ends.
    ends: [usize; LISTS],
}

impl Default for Pool {
    fn default() -> Self {
        Pool::new()
    }
}

impl Clone for Pool {
    /// A copy with the whole pool's capacity reserved again (a derived clone would keep only
    /// the length).
    fn clone(&self) -> Self {
        let mut p = Pool::new();
        p.nodes.extend_from_slice(&self.nodes);
        p.ends = self.ends;
        p
    }
}

impl std::ops::Index<usize> for Pool {
    type Output = [Event];
    fn index(&self, k: usize) -> &[Event] {
        self.list(k)
    }
}

impl Pool {
    /// An empty pool with the capacity of [`POOL_NODES`] nodes.
    pub fn new() -> Self {
        Pool {
            nodes: Vec::with_capacity(POOL_NODES),
            ends: [0; LISTS],
        }
    }

    fn start(&self, k: usize) -> usize {
        if k == 0 { 0 } else { self.ends[k - 1] }
    }

    /// List k.
    pub fn list(&self, k: usize) -> &[Event] {
        &self.nodes[self.start(k)..self.ends[k]]
    }

    fn list_mut(&mut self, k: usize) -> &mut [Event] {
        let s = self.start(k);
        &mut self.nodes[s..self.ends[k]]
    }

    /// The nodes of all lists.
    pub fn used(&self) -> usize {
        self.nodes.len()
    }

    /// The nodes free (`[pool+4]`, what `sub_10014434` reads).
    pub fn free(&self) -> usize {
        POOL_NODES.saturating_sub(self.nodes.len())
    }

    /// Node `e` at index `at` of list k.
    fn insert(&mut self, k: usize, at: usize, e: Event) {
        let g = self.start(k) + at;
        self.nodes.insert(g, e);
        for end in &mut self.ends[k..] {
            *end += 1;
        }
    }

    /// Removes the node at index `at` of list k.
    fn remove(&mut self, k: usize, at: usize) {
        let g = self.start(k) + at;
        self.nodes.remove(g);
        for end in &mut self.ends[k..] {
            *end -= 1;
        }
    }

    /// Keeps the nodes of list k for which `keep(index, node)` holds, in their order, and
    /// returns how many nodes before index `before` it removed. One pass, so a removal of
    /// many nodes moves the nodes after the list once.
    fn retain(&mut self, k: usize, before: usize, mut keep: impl FnMut(&Event) -> bool) -> usize {
        let (s, e) = (self.start(k), self.ends[k]);
        let mut w = s;
        let mut gone_before = 0;
        for r in s..e {
            if keep(&self.nodes[r]) {
                self.nodes[w] = self.nodes[r];
                w += 1;
            } else if r - s < before {
                gone_before += 1;
            }
        }
        let n = e - w;
        self.nodes.drain(w..e);
        for end in &mut self.ends[k..] {
            *end -= n;
        }
        gone_before
    }

    /// This pool becomes a copy of `src`, every list (the recordings too), within its capacity:
    /// no allocation when `src` fits, as a pool always does (the snapshot of orb's Undo of the
    /// Initialize commands, task-306).
    pub fn copy_from(&mut self, src: &Pool) {
        self.nodes.clear();
        self.nodes.extend_from_slice(&src.nodes);
        self.ends = src.ends;
    }

    /// Empties list k.
    fn clear(&mut self, k: usize) {
        let (s, e) = (self.start(k), self.ends[k]);
        self.nodes.drain(s..e);
        for end in &mut self.ends[k..] {
            *end -= e - s;
        }
    }

    /// This pool becomes a copy of `src`'s track lists (0..9), with empty recording lists,
    /// within its capacity (the callback's copy for the UI, `orb_player::Tracks`).
    pub fn copy_tracks_from(&mut self, src: &Pool) {
        let n = src.ends[TRACKS - 1];
        self.nodes.clear();
        self.nodes.extend_from_slice(&src.nodes[..n]);
        self.ends[..TRACKS].copy_from_slice(&src.ends[..TRACKS]);
        self.ends[TRACKS..].fill(n);
    }
}

/// `dbl_10153808`: the position step per block per BPM*1000, `64*192 / (60*44100*1000)`.
pub const TEMPO_STEP: f64 = f64::from_bits(0x3ED3_7A72_81B2_2126); // 4.6439909297052154e-06
/// `dbl_10153810`: the song ends when the position passes this (999 bars minus a tick).
pub const SONG_END: f64 = f64::from_bits(0x4127_69FE_0000_0000); // 767231.0
/// `sub_10014750` drops an event whose quantised tick is above this (`cmp eax, 0BB4FFh`).
pub const LAST_EVENT_TICK: i32 = 767_231;
/// Ticks per bar: stop relocates to `768 * (tick / 768)` (`sub_1001E100`).
const BAR: i32 = song::TICKS_PER_BAR as i32;
/// The shuffle period: one 8th note (`mov edx, 60h` at 0x1001F3E8).
const SHUFFLE_PERIOD: i32 = 96;
/// The shuffle amount's divisor (`mov ecx, 6` at 0x1001F3D7).
const SHUFFLE_DIV: i32 = 6;

/// `trunc(pos)`: `fistp qword` under a truncating control word, of which the low dword is used
/// (0x1001EF88, `sub_1001F870`).
pub fn trunc(pos: f64) -> i32 {
    pos as i64 as i32
}

/// The shuffled tick of 0x1001F3D0..0x1001F429: `96*(t/96) + max(0, t%96 - shuffle/6)`, with
/// `idiv`'s truncating division. Only the second 16th of each 8th moves, by up to 21 ticks.
pub fn shuffled(tick: i32, shuffle: u8) -> i32 {
    let s = shuffle as i32 / SHUFFLE_DIV;
    let r = (tick % SHUFFLE_PERIOD - s).max(0);
    SHUFFLE_PERIOD * (tick / SHUFFLE_PERIOD) + r
}

/// One node of a track's list: `+0` next, `+4` tick, `+8` controller, `+9` value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Event {
    pub tick: i32,
    pub ctrl: u8,
    pub value: u8,
}

/// The new nodes a merge holds before it moves them into the track's list in one pass
/// ([`Track::merge`]); a merge of more moves them in chunks of this many.
pub const MERGE_CHUNK: usize = 4096;

/// A merge's new nodes, in the list's order (tick ascending; of one tick, the latest insert
/// first), in a buffer of [`MERGE_CHUNK`] nodes reserved with the [`Sequencer`], so that the
/// merge, which runs in the audio callback, never allocates. Empty between merges.
#[derive(Debug)]
struct MergeBuf(Vec<Event>);

impl Default for MergeBuf {
    fn default() -> Self {
        MergeBuf(Vec::with_capacity(MERGE_CHUNK))
    }
}

impl Clone for MergeBuf {
    /// A new buffer with the whole capacity (it holds nothing between merges).
    fn clone(&self) -> Self {
        MergeBuf::default()
    }
}

impl PartialEq for MergeBuf {
    /// Scratch space, not state.
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// Where [`Track::find`] found a node.
enum Found {
    /// In the merge's buffer, at this index.
    Pending(usize),
    /// In the pool, at this index of `nodes`.
    Node(usize),
}

/// One track: the events of one device. Its two lists are in the song's [`Pool`]: list
/// `device.id()` and its recording, list `9 + device.id()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Track {
    /// `+0x104`: the device whose set-param (slot 11) the events go to.
    pub device: Device,
    /// Its list in the pool (after the head, in list order); the recording list is
    /// `TRACKS + list`.
    list: usize,
    /// The index of the node after the one `+0x228` points at (0 when it points at the head).
    next: usize,
    /// `+4`: a controller whose byte is set is skipped by the clock's apply loops (not by the
    /// chase). Zero from the ctor; set by the record hook (0x10021402) for the controller it
    /// records and cleared for all by the merge (`sub_1001F520`, 0x1001F767), so a controller
    /// moved by hand while recording keeps its value against the track until the next merge.
    pub hold: [bool; 128],
    /// List `+0xD`: each controller's quantisation in 24-tick units, 1 from the list ctor. The
    /// recording list's (`+0x198`) stays 1 (the 303's slot 7, `sub_10021560`, sets only the
    /// first list's).
    quant: [u8; 128],
}

/// A track with the song's pool: what a device's slots 7 and 8 get from the song ctor and
/// from song msgs 28 and 29 ([`crate::frame::SongDevice::init_track`],
/// [`crate::frame::SongDevice::insert_pattern`]).
pub struct TrackMut<'a> {
    pub track: &'a mut Track,
    pub pool: &'a mut Pool,
}

impl TrackMut<'_> {
    /// Slot 8's insert, `sub_1001F900` → `sub_10014750`: [`Track::insert`] taking a node only
    /// while the pool has more than one free (`sub_10014434`; on the song ctor's empty pool the
    /// check always passes).
    pub fn insert(&mut self, tick: i32, ctrl: u8, value: u8) -> bool {
        self.track.insert_pooled(self.pool, tick, ctrl, value)
    }

    /// [`Track::set_quant`].
    pub fn set_quant(&mut self, ctrl: u8, q: u8) {
        self.track.set_quant(ctrl, q);
    }

    /// [`Track::events`].
    pub fn events(&self) -> &[Event] {
        self.track.events(self.pool)
    }
}

/// What [`insert_into`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Inserted {
    /// A new node at this index.
    At(usize),
    /// A node of the same tick and controller took the value.
    Replaced,
    /// Above [`LAST_EVENT_TICK`], or no node free in the pool.
    Dropped,
}

/// `sub_10014750` on list `k` of the pool, of quantisation `quant`: the tick rounded up to a
/// multiple of `24*q` (`sub_100146E4`), dropped above [`LAST_EVENT_TICK`]; a node of the same
/// tick and controller takes the value, else a new node (`pooled`: when the pool has more than
/// one free, `sub_10014434`) goes before the first node whose tick is not below it.
fn insert_into(
    pool: &mut Pool,
    k: usize,
    quant: &[u8; 128],
    tick: i32,
    ctrl: u8,
    value: u8,
    pooled: bool,
) -> Inserted {
    let t = round_up(tick, quant_of(quant, ctrl));
    if t > LAST_EVENT_TICK {
        return Inserted::Dropped;
    }
    let list = pool.list_mut(k);
    let at = list.partition_point(|e| t > e.tick);
    if let Some(e) = list[at..]
        .iter_mut()
        .take_while(|e| e.tick == t)
        .find(|e| e.ctrl == ctrl)
    {
        e.value = value;
        return Inserted::Replaced;
    }
    // 0x1001443A: cmp [pool+4], 1; jle (no node)
    if pooled && pool.free() <= 1 {
        return Inserted::Dropped;
    }
    pool.insert(
        k,
        at,
        Event {
            tick: t,
            ctrl,
            value,
        },
    );
    Inserted::At(at)
}

/// The quantisation unit of `ctrl`: `24*q` ticks. A controller above 127 reads past the
/// 128-byte array in the DLL; no message the panel sends records one (orb records none,
/// [`Sequencer::record_hook`]), and 1 stands in.
fn quant_of(quant: &[u8; 128], ctrl: u8) -> i32 {
    24 * quant.get(ctrl as usize).copied().unwrap_or(1) as i32
}

/// `sub_100146E4`: up to a multiple of `unit` (`lea ebx, [edx+eax*8-1]; cdq; idiv; imul`).
fn round_up(tick: i32, unit: i32) -> i32 {
    tick.wrapping_add(unit).wrapping_sub(1) / unit * unit
}

/// `sub_10014720`: down to a multiple of `unit` (`cdq; idiv; imul`, truncating).
fn round_down(tick: i32, unit: i32) -> i32 {
    tick / unit * unit
}

impl Track {
    /// The track of `device`, whose lists are lists `device.id()` and `9 + device.id()` of the
    /// song's pool.
    pub fn new(device: Device) -> Self {
        Track {
            device,
            list: device.id() as usize,
            next: 0,
            hold: [false; 128],
            quant: [1; 128],
        }
    }

    /// The track's list in `pool`.
    pub fn events<'p>(&self, pool: &'p Pool) -> &'p [Event] {
        pool.list(self.list)
    }

    /// The recording list (`+0x198`) in `pool`: what the record hook took since the last merge.
    pub fn recorded<'p>(&self, pool: &'p Pool) -> &'p [Event] {
        pool.list(self.rec())
    }

    /// The recording list's index in the pool.
    fn rec(&self) -> usize {
        TRACKS + self.list
    }

    /// The index of the next event the clock applies.
    pub fn next(&self) -> usize {
        self.next
    }

    /// `sub_100146E4`: the tick rounded up to a multiple of `24*q`. q is 1 from the list ctor, so
    /// a loaded tick, already a multiple of 24, is unchanged, except on the 303 tracks, whose
    /// controllers 0 and 1 have q = 32 (a bar) from the 303's slot 7 ([`Track::set_quant`]).
    fn quantise(&self, tick: i32, ctrl: u8) -> i32 {
        round_up(tick, quant_of(&self.quant, ctrl))
    }

    /// `sub_10014600(list, ctrl, q)`: controller `ctrl`'s quantisation becomes `q` 24-tick units
    /// (the 303's slot 7 sets 32 for controllers 0 and 1).
    pub fn set_quant(&mut self, ctrl: u8, q: u8) {
        self.quant[ctrl as usize & 127] = q;
    }

    /// `sub_10014750`, the insert of song msg 37 (`sub_10021010`): the tick is quantised and an
    /// event above [`LAST_EVENT_TICK`] is dropped. The new node goes before the first node whose
    /// tick is not below it, unless a node of the same tick has the same controller, whose value
    /// is then replaced. So events of one tick end up in reverse arrival order. Returns whether
    /// the event was kept. The node is taken from `pool` without the free check (the tests'
    /// inserts; the song ctor's and every other path's go through the checked
    /// [`TrackMut::insert`] and [`Sequencer::add_events`]). Being unchecked, it can grow the
    /// pool past [`POOL_NODES`] (the pool's `Vec` then reallocates); only the tests call it,
    /// never the callback.
    pub fn insert(&mut self, pool: &mut Pool, tick: i32, ctrl: u8, value: u8) -> bool {
        self.insert_to(pool, tick, ctrl, value, false)
    }

    /// [`Track::insert`] taking a node only while the pool has more than one free.
    fn insert_pooled(&mut self, pool: &mut Pool, tick: i32, ctrl: u8, value: u8) -> bool {
        self.insert_to(pool, tick, ctrl, value, true)
    }

    fn insert_to(&mut self, pool: &mut Pool, tick: i32, ctrl: u8, value: u8, pooled: bool) -> bool {
        match insert_into(pool, self.list, &self.quant, tick, ctrl, value, pooled) {
            Inserted::At(at) => {
                // a node inserted before the applied one moves it; one right after it becomes
                // its next
                if at < self.next {
                    self.next += 1;
                }
                true
            }
            Inserted::Replaced => true,
            Inserted::Dropped => false,
        }
    }

    /// The list part of the record hook `sub_10021340` (0x100213EF..0x10021506), for a
    /// controller below 128: the hold byte is set; the tick of the position, `trunc(pos)`, is
    /// rounded to the controller's quantisation of the first list, **up** while playing
    /// (`sub_100146E0` → `sub_100146E4`) and **down** when stopped (`sub_10014720`); with the
    /// loop on, a tick equal to the loop end becomes the loop start; then it is inserted into
    /// the recording list (`sub_10014750` on `+0x198`, whose own quantisation is 1).
    fn record(
        &mut self,
        pool: &mut Pool,
        now: i32,
        playing: bool,
        wrap: Option<(i32, i32)>,
        ctrl: u8,
        value: u8,
    ) {
        self.hold[ctrl as usize & 127] = true;
        let unit = quant_of(&self.quant, ctrl);
        let mut t = if playing {
            round_up(now, unit)
        } else {
            round_down(now, unit)
        };
        // 0x100214B9: loop on and t == loop end (+0x10) → loop start (+0xC)
        if let Some((start, end)) = wrap
            && t == end
        {
            t = start;
        }
        insert_into(pool, self.rec(), &[1; 128], t, ctrl, value, true);
    }

    /// `sub_1001F520(track, t)`, the merge of the recording into the track:
    /// - the first list is walked while its tick is at most `t`; every recorded node whose tick is
    ///   at most the walked node's marks its controller; the walked node's value is the
    ///   controller's value so far, and a walked node of a marked controller is removed
    ///   (`sub_10014B30`) unless its tick is 0; the recorded nodes up to `t` mark theirs too
    ///   (0x1001F649);
    /// - every marked controller `c`, ascending, gets its value so far back at `round_up(t + 1)` of
    ///   its quantisation unless the track has a node of `c` there (`sub_10014840`): after the
    ///   recorded span the controller returns to the track's automation (0xFF, the value of a
    ///   controller with no node up to `t`, goes in as it is);
    /// - every recorded node up to `t` is inserted into the first list (a node of the same tick and
    ///   controller takes its value);
    /// - the whole recording list is freed, the nodes above `t` too (`sub_100149A0`, OB-18), and
    ///   the hold mask is cleared (0x1001F767).
    ///
    /// The applied pointer `+0x228` is not touched: when its own node is removed it dangles in
    /// the DLL (OB-19); here it moves to the node before. Every caller but record off in
    /// pattern mode chases right after, which sets it again.
    ///
    /// How orb does it (task-084): the result is the DLL's insert by insert, node for node, but
    /// the pool's nodes after the list move a few times per track, not once per node (which
    /// cost 1.8 ms for 1000 recorded nodes on a 10000-node song). The walk keeps its nodes in
    /// one pass and frees the removed ones at once; the returns and the recorded nodes go
    /// through [`Track::add`], which applies `sub_10014750`'s rules (the quantisation, the
    /// last tick, a node of the same tick and controller taking the value, the free check
    /// counting the nodes taken so far) against the list and the new nodes held in `buf`;
    /// [`Track::flush`] then moves them into the list.
    fn merge(&mut self, pool: &mut Pool, buf: &mut MergeBuf, t: i32) {
        let (ev, rec) = (self.list, self.rec());
        if pool.list(rec).is_empty() {
            // nothing marked, removed or inserted
            self.hold = [false; 128];
            return;
        }
        let mut marked = [false; 128];
        let mut value = [0xFFu8; 128];
        let (s, end) = (pool.start(ev), pool.ends[ev]);
        let (rs, rn) = (pool.start(rec), pool.list(rec).len());
        // the applied node's index before the walk: a removed node before it moves it back
        let applied = s + self.next;
        let mut r = 0;
        let (mut i, mut w) = (s, s);
        while i < end && t >= pool.nodes[i].tick {
            let m = pool.nodes[i];
            while r < rn && pool.nodes[rs + r].tick <= m.tick {
                marked[pool.nodes[rs + r].ctrl as usize & 127] = true;
                r += 1;
            }
            let c = m.ctrl as usize & 127;
            value[c] = m.value;
            if marked[c] && m.tick != 0 {
                // the node goes back to the pool
                if i < applied {
                    self.next -= 1;
                }
            } else {
                pool.nodes[w] = m;
                w += 1;
            }
            i += 1;
        }
        if w < i {
            pool.nodes.drain(w..i);
            for end in &mut pool.ends[ev..] {
                *end -= i - w;
            }
        }
        let recorded = pool.list(rec);
        while r < recorded.len() && t >= recorded[r].tick {
            marked[recorded[r].ctrl as usize & 127] = true;
            r += 1;
        }
        buf.0.clear();
        // 0x1001F675: the return to the automation, controllers ascending
        for c in 0..128u8 {
            if !marked[c as usize] {
                continue;
            }
            let back = self.quantise(t.wrapping_add(1), c);
            if self.find(pool, buf, back, c).is_none() {
                self.add(pool, buf, back, c, value[c as usize]);
            }
        }
        // 0x1001F722: the recorded nodes up to t, in list order; their nodes stay taken until
        // the whole recording list is freed after them
        for k in 0..r {
            let e = pool.list(rec)[k];
            self.add(pool, buf, e.tick, e.ctrl, e.value);
        }
        self.flush(pool, buf);
        pool.clear(rec);
        self.hold = [false; 128];
    }

    /// The node of tick `t` and controller `ctrl` in the list as the merge has it so far: the
    /// new nodes in `buf` first (of one tick they go before the list's), then the list's.
    fn find(&self, pool: &Pool, buf: &MergeBuf, t: i32, ctrl: u8) -> Option<Found> {
        let of_tick = |l: &[Event]| {
            let at = l.partition_point(|e| t > e.tick);
            l[at..]
                .iter()
                .take_while(|e| e.tick == t)
                .position(|e| e.ctrl == ctrl)
                .map(|j| at + j)
        };
        if let Some(j) = of_tick(&buf.0) {
            return Some(Found::Pending(j));
        }
        of_tick(pool.list(self.list)).map(|j| Found::Node(pool.start(self.list) + j))
    }

    /// [`Track::insert_pooled`] during a merge: a new node goes into `buf` (before the nodes
    /// of its tick, as `sub_10014750` puts it before the list's), its pool node counted as
    /// taken; a full `buf` is flushed first.
    fn add(&mut self, pool: &mut Pool, buf: &mut MergeBuf, tick: i32, ctrl: u8, value: u8) {
        let t = round_up(tick, quant_of(&self.quant, ctrl));
        if t > LAST_EVENT_TICK {
            return;
        }
        match self.find(pool, buf, t, ctrl) {
            Some(Found::Pending(j)) => buf.0[j].value = value,
            Some(Found::Node(j)) => pool.nodes[j].value = value,
            None => {
                // 0x1001443A: cmp [pool+4], 1; jle (no node), the held nodes taken
                if POOL_NODES.saturating_sub(pool.nodes.len() + buf.0.len()) <= 1 {
                    return;
                }
                if buf.0.len() == MERGE_CHUNK {
                    self.flush(pool, buf);
                }
                let at = buf.0.partition_point(|e| t > e.tick);
                buf.0.insert(
                    at,
                    Event {
                        tick: t,
                        ctrl,
                        value,
                    },
                );
            }
        }
    }

    /// Moves the new nodes of `buf` into the list: the pool's nodes after the list move once,
    /// then the list and `buf` merge from the back, a new node before the list's nodes of its
    /// tick. The applied node keeps its place: the new nodes of a tick up to its tick go
    /// before it (`sub_10014750` puts a node before the first of a tick not below it).
    fn flush(&mut self, pool: &mut Pool, buf: &mut MergeBuf) {
        let n = buf.0.len();
        if n == 0 {
            return;
        }
        let (s, e, used) = (
            pool.start(self.list),
            pool.ends[self.list],
            pool.nodes.len(),
        );
        if self.next > 0 {
            let p = pool.nodes[s + self.next - 1].tick;
            self.next += buf.0.partition_point(|x| x.tick <= p);
        }
        // within the capacity: every new node found more than one free
        pool.nodes.resize(
            used + n,
            Event {
                tick: 0,
                ctrl: 0,
                value: 0,
            },
        );
        pool.nodes.copy_within(e..used, e + n);
        let (mut i, mut j, mut w) = (e, n, e + n);
        while j > 0 {
            w -= 1;
            if i > s && pool.nodes[i - 1].tick >= buf.0[j - 1].tick {
                i -= 1;
                pool.nodes[w] = pool.nodes[i];
            } else {
                j -= 1;
                pool.nodes[w] = buf.0[j];
            }
        }
        for end in &mut pool.ends[self.list..] {
            *end += n;
        }
        buf.0.clear();
    }

    /// The chase `sub_100151F0(list, state, p)`: the value of the last event at or before `p` for
    /// each controller (0xFF for none), and the applied pointer moved to that event.
    pub fn chase(&mut self, pool: &Pool, p: i32) -> [u8; 128] {
        let (state, i) = self.state_at(pool, p);
        self.next = i;
        state
    }

    /// `sub_100151F0` into a buffer of its own, the applied pointer untouched: the state at `p`
    /// and the index of the first node after `p`.
    fn state_at(&self, pool: &Pool, p: i32) -> ([u8; 128], usize) {
        let events = self.events(pool);
        let mut state = [0xFF; 128];
        let mut i = 0;
        while i < events.len() && p >= events[i].tick {
            let e = events[i];
            state[e.ctrl as usize & 127] = e.value;
            i += 1;
        }
        (state, i)
    }

    /// `sub_10014910(list, start, end)`: every node with `start <= tick < end` is unlinked and
    /// freed, except the nodes at tick 0 (the walk steps over a node whose tick is 0,
    /// 0x10014945). The applied pointer is not touched in the DLL (it dangles when its node
    /// goes; every caller chases right after in song mode, `sub_1001FDC0`); here it moves back
    /// over the removed nodes before it.
    fn remove_range(&mut self, pool: &mut Pool, start: i32, end: i32) {
        let gone = pool.retain(self.list, self.next, |e| {
            e.tick == 0 || e.tick < start || e.tick >= end
        });
        self.next -= gone;
    }

    /// `sub_10020350(track, t)`: the track's state at `t` (`sub_100151F0` into a local buffer)
    /// is inserted at `t` (`sub_10015240`: for each controller 0..127 that has a value, the
    /// tick quantised by `sub_100146E4`, dropped above [`LAST_EVENT_TICK`], a node of the same
    /// tick and controller taking the value, else a new node from the pool before the tick's
    /// first node); with record on (`record`, global byte 2, 0x100203D7) the same state goes
    /// into the recording list too (`+0x198`, quantisation 1), after the first list's.
    fn insert_state(&mut self, pool: &mut Pool, t: i32, record: bool) {
        let (state, _) = self.state_at(pool, t);
        for (c, &v) in state.iter().enumerate() {
            if v != 0xFF {
                self.insert_pooled(pool, t, c as u8, v);
            }
        }
        if record {
            for (c, &v) in state.iter().enumerate() {
                if v != 0xFF {
                    insert_into(pool, self.rec(), &[1; 128], t, c as u8, v, true);
                }
            }
        }
    }

    /// The clock's apply loop (0x1001F2C6, 0x1001F032, 0x1001FADB): every event after the applied
    /// one with `tick <= t`, in list order, skipping held controllers.
    pub fn advance(&mut self, pool: &Pool, t: i32, mut apply: impl FnMut(Event)) {
        let events = pool.list(self.list);
        while self.next < events.len() && t >= events[self.next].tick {
            let e = events[self.next];
            if !self.hold[e.ctrl as usize & 127] {
                apply(e);
            }
            self.next += 1;
        }
    }

    /// `sub_100149F0`, run on every track by stop: an event whose value equals the previous
    /// value of its controller (0xFF before the first) is removed, unless it is at tick 0.
    pub fn prune(&mut self, pool: &mut Pool) {
        let mut last = [0xFFu8; 128];
        let mut i = 0;
        while i < pool.list(self.list).len() {
            let e = pool.list(self.list)[i];
            let c = e.ctrl as usize & 127;
            if e.value != last[c] {
                last[c] = e.value;
                i += 1;
            } else if e.tick != 0 {
                pool.remove(self.list, i);
                // the DLL frees the node; the applied pointer then moves back to its predecessor
                if i < self.next {
                    self.next -= 1;
                }
            } else {
                i += 1;
            }
        }
    }
}

/// The song's sequencer and transport state.
#[derive(Debug, Clone, PartialEq)]
pub struct Sequencer {
    /// `seq+0`: the position in ticks.
    pub pos: f64,
    /// `seq+8`: the tick of the last block that did work.
    pub last_tick: i32,
    /// `seq+0x10`: the song-mode tick saved when pattern mode was entered.
    pub saved_tick: i32,
    /// The tracks of devices 0..8 (`sub_1001EEF0(i, i, n)` in the song ctor).
    pub tracks: [Track; TRACKS],
    /// The nodes of the tracks' lists (`dword_10100230`, the module doc).
    pub pool: Pool,
    /// `song+0x158`: song mode, play, record, loop, tempo, loop points, shuffle. The sound-model
    /// byte (`+0x16D`) is the frame's `vintage`.
    pub globals: Globals,
    /// Not in the original: counts the changes of the tracks' event lists after the song ctor
    /// (a msg 37 batch, a merge, a stop's prune), so that a reader knows when to copy them.
    pub revision: u64,
    /// Not in the original: the merges that moved recorded events into the tracks.
    pub merges: u64,
    /// Not in the original: the merge's new nodes ([`MergeBuf`]).
    buf: MergeBuf,
}

impl Default for Sequencer {
    /// The ctors' values: `sub_1001EC50` (position, ticks 0) and the global struct's
    /// `sub_10018AC0` (tempo 120000, loop end 768, shuffle 64, byte 22 = 1, the rest 0).
    fn default() -> Self {
        Sequencer {
            pos: 0.0,
            last_tick: 0,
            saved_tick: 0,
            tracks: Device::ALL.map(Track::new),
            pool: Pool::new(),
            globals: Globals {
                song_mode: 0,
                play: 0,
                byte2: 0,
                loop_on: 0,
                tempo: 120_000,
                locate: 0,
                loop_start: 0,
                loop_end: 768,
                shuffle: 64,
                vintage: 0,
                byte22: 1,
            },
            revision: 0,
            merges: 0,
            buf: MergeBuf::default(),
        }
    }
}

impl Sequencer {
    fn song_mode(&self) -> bool {
        self.globals.song_mode != 0
    }

    fn playing(&self) -> bool {
        self.globals.play != 0
    }

    /// The tick of the position (`sub_1001F870`).
    pub fn tick_pos(&self) -> i32 {
        trunc(self.pos)
    }

    /// Track `i`'s list.
    pub fn events(&self, i: usize) -> &[Event] {
        self.tracks[i].events(&self.pool)
    }

    /// Track `i`'s recording list.
    pub fn recorded(&self, i: usize) -> &[Event] {
        self.tracks[i].recorded(&self.pool)
    }

    /// Track `i` with the pool, to insert into.
    pub fn track_mut(&mut self, i: usize) -> TrackMut<'_> {
        TrackMut {
            track: &mut self.tracks[i],
            pool: &mut self.pool,
        }
    }

    /// Chases every track to `p` and applies each controller that has a value, ascending, to
    /// its device (the loop at 0x1001FA2C; `sub_10012B40` finds the device by id).
    fn chase_all(&mut self, p: i32, devices: &mut Devices) {
        for tr in &mut self.tracks {
            let state = tr.chase(&self.pool, p);
            let d = devices.device_mut(tr.device);
            for (c, &v) in state.iter().enumerate() {
                if v != 0xFF {
                    d.set_param(c as u8, v);
                }
            }
        }
    }

    /// The apply loop over every track, in track order.
    fn advance_all(&mut self, t: i32, devices: &mut Devices) {
        for tr in &mut self.tracks {
            let d = devices.device_mut(tr.device);
            tr.advance(&self.pool, t, |e| d.set_param(e.ctrl, e.value));
        }
    }

    /// `sub_1001EF20`, the clock, once per block before the devices render:
    /// ```text
    /// if !play: return
    /// inc = tempo * TEMPO_STEP; pos = pos + inc; t = trunc(pos)
    /// if t > lastTick:
    ///     if song mode:
    ///         if loop on and loopEnd <= pos and !(loopEnd <= pos - inc):
    ///             apply events up to loopEnd - 1
    ///             pos = pos + (loopStart - loopEnd); t = trunc(pos)
    ///             chase to loopStart, relocate every device to loopStart
    ///         apply events up to t
    ///         if pos > 767231: song end (stop, locate 0), return
    ///     every device: step(t, shuffled(t), tempo)
    /// lastTick = t
    /// ```
    /// The tempo of the position step is the current song's (`sub_1001BCC0` without MIDI sync),
    /// which is this song's whenever it renders. Returns whether it reached the step loop,
    /// whose tempo argument it stores in a local at 0x1001F3B3 (`esp+0x34`, the address of
    /// [`crate::frame::Frame::stack_word`]).
    pub fn tick(&mut self, devices: &mut Devices) -> bool {
        if !self.playing() {
            return false;
        }
        let tempo = self.globals.tempo as i32;
        // 0x1001EF51: fild tempo; fmul; fld pos; fadd st, st(1)
        let inc = tempo as f64 * TEMPO_STEP;
        self.pos += inc;
        let mut t = trunc(self.pos);
        if t > self.last_tick {
            if self.song_mode() {
                let le = self.globals.loop_end as i32;
                // 0x1001EFD6: loopEnd <= pos; 0x1001EFF3: loopEnd > pos - inc
                if self.globals.loop_on != 0 && le as f64 <= self.pos && le as f64 > self.pos - inc
                {
                    self.advance_all(le.wrapping_sub(1), devices);
                    // 0x1001F0C7: with record on, every track's recording is merged up to
                    // loopEnd - 1 (`sub_1001F520`, 0x1001F0FD)
                    if self.globals.byte2 != 0 {
                        self.merge_all(le.wrapping_sub(1));
                    }
                    let ls = self.globals.loop_start as i32;
                    // 0x1001F135: fld pos; fild (start - end); faddp
                    self.pos += ls.wrapping_sub(le) as f64;
                    t = trunc(self.pos);
                    // 0x1001F182 tests song mode again, which holds here
                    self.chase_all(ls, devices);
                    devices.for_each(|d| d.relocate(ls, t));
                }
                self.advance_all(t, devices);
                // 0x1001F35F: pos > 767231
                if self.pos > SONG_END {
                    self.song_end(devices);
                    return false;
                }
            }
            let sp = shuffled(t, self.globals.shuffle);
            let tempo = self.globals.tempo as i32;
            devices.for_each(|d| d.step(t, sp, tempo));
            self.last_tick = t;
            return true;
        }
        self.last_tick = t;
        false
    }

    /// 0x1001F484: record off (`sub_10021194(0)`, with its merge and chase), the host is told
    /// (msg 35 `{2,0}` and `{1,0}`), stop (`sub_100218A0`, without the bar locate) and locate
    /// to 0.
    fn song_end(&mut self, devices: &mut Devices) {
        self.set_record(0, devices);
        self.halt(devices);
        self.locate(0, devices);
    }

    /// `sub_1001F984`, locate (song msg 35 `{8,p}`, msg 32): a negative tick becomes 0; in song
    /// mode every track is chased to it (with record on, stopped and a later tick, the events up
    /// to it are applied instead); then the position and the last tick become it and every
    /// device is relocated (slot 2).
    pub fn locate(&mut self, p: i32, devices: &mut Devices) {
        let p = p.max(0);
        if self.globals.byte2 != 0 && !self.playing() && p > self.tick_pos() {
            self.advance_all(p, devices);
        } else if self.song_mode() {
            self.chase_all(p, devices);
        }
        self.pos = p as f64;
        devices.for_each(|d| d.relocate(p, p));
        self.last_tick = p;
    }

    /// `sub_1001F8B0`, the position alone (`fild [esp+arg_0]; fstp qword ptr [ecx]`): no chase,
    /// no relocate, the last tick kept. The TopHat host sync (`sub_1001C2F4`, 0x1001C38F) sets
    /// the position so to the host's tick in each audio call while the host does not jump
    /// (upstream's plugin document (removed, task-231) section 8). The original stores an integer
    /// tick; orb's plugin passes the host's fractional tick.
    pub fn set_pos(&mut self, pos: f64) {
        self.pos = pos;
    }

    /// Slot 2 (relocate) on one device, at the current tick, as a locate does to every device
    /// (`sub_1001F984`): the device's pending pattern and on/off switch take effect at once.
    /// Not a path of the original's: the plugin's pattern selection by notes (task-198).
    pub fn relocate_device(&self, d: Device, devices: &mut Devices) {
        let p = self.tick_pos();
        devices.device_mut(d).relocate(p, p);
    }

    /// `sub_100218A0`: when playing, clear the play flag and stop every device (slot 4); prune
    /// every track; in song mode chase every track to the current tick.
    pub fn halt(&mut self, devices: &mut Devices) {
        if self.playing() {
            self.globals.play = 0;
            devices.for_each(|d| d.stop());
        }
        for tr in &mut self.tracks {
            tr.prune(&mut self.pool);
        }
        self.revision += 1;
        if self.song_mode() {
            let t = self.tick_pos();
            self.chase_all(t, devices);
        }
    }

    /// Song msg 35 `{1,0}`, `sub_1001E100` (no ReWire, no MIDI sync): `sub_100218A0`, then
    /// locate to the start of the current bar.
    pub fn stop(&mut self, devices: &mut Devices) {
        self.halt(devices);
        let t = self.tick_pos();
        self.locate(BAR * (t / BAR), devices);
    }

    /// Song msg 35 `{1,1}`, `sub_1001BE74` → 0x1001F7E4: when stopped, set the play flag and
    /// start every device (slot 3). The position is not touched.
    pub fn play(&mut self, devices: &mut Devices) {
        if !self.playing() {
            self.globals.play = 1;
            let now = self.tick_pos();
            devices.for_each(|d| d.start(now));
        }
    }

    /// Song msg 35 `{0,m}`, `sub_1001FBE0`: on a change, store the byte and tell every device
    /// (slot 9, `sub_10013FB0`); then song mode locates to the saved tick, pattern mode relocates
    /// every device to 0 and saves the current tick.
    pub fn set_song_mode(&mut self, mode: u8, devices: &mut Devices) {
        if self.globals.song_mode != mode {
            self.globals.song_mode = mode;
            devices.for_each(|d| d.set_song_mode(mode != 0));
        }
        if mode != 0 {
            self.locate(self.saved_tick, devices);
        } else {
            let now = self.tick_pos();
            devices.for_each(|d| d.relocate(0, now));
            self.saved_tick = self.tick_pos();
        }
    }

    /// Song msg 35 `{2,v}` (and msg 36's byte 2, and the song end), `sub_10021194`: turning
    /// record off (`v == 0`) while it is on merges every track's recording up to the current
    /// tick (`sub_1001F520`, 0x10021206) and, in song mode, chases every track to that tick
    /// (0x10021294); then the byte is stored as it is.
    pub fn set_record(&mut self, v: u8, devices: &mut Devices) {
        if self.globals.byte2 != 0 && v == 0 {
            let t = self.tick_pos();
            self.merge_all(t);
            if self.song_mode() {
                self.chase_all(t, devices);
            }
        }
        self.globals.byte2 = v;
    }

    /// `sub_1001FD70`, song msg 28's first step: every node of every track is removed except
    /// those at tick 0 (`sub_10014910(0, 0x7FFFFFFF)` on each track's first list; the
    /// recording lists are not touched).
    pub fn clear_tracks(&mut self) {
        for tr in &mut self.tracks {
            tr.remove_range(&mut self.pool, 0, i32::MAX);
        }
        self.revision += 1;
    }

    /// Song msg 29's first steps (0x1001244A..0x1001247E): `sub_10021520(end)` inserts each
    /// track's state at `end` there (`sub_10020350`, into the recording list too with record
    /// on), then `sub_100208A0(start, end)` removes every node of `start <= tick < end` but those
    /// at tick 0 from every track (`sub_10014910`). So after the range the song goes on as it
    /// did.
    pub fn clear_range(&mut self, start: i32, end: i32) {
        let record = self.globals.byte2 != 0;
        for tr in &mut self.tracks {
            tr.insert_state(&mut self.pool, end, record);
        }
        for tr in &mut self.tracks {
            tr.remove_range(&mut self.pool, start, end);
        }
        self.revision += 1;
    }

    /// `sub_1001FDC0(seq, trunc(pos))`, the last step of song msgs 28 and 29: in song mode every
    /// track is chased to the current tick and each controller that has a value is applied,
    /// ascending (`sub_10012B40`), as a locate's chase; the position, the last tick and the
    /// devices' relocate are not touched. Nothing in pattern mode.
    pub fn refresh(&mut self, devices: &mut Devices) {
        if self.song_mode() {
            let t = self.tick_pos();
            self.chase_all(t, devices);
        }
    }

    /// Not in the original: orb's Undo of Edit's Initialize commands (task-306). Every list
    /// becomes `pool`'s (an O(1) swap; `pool` gets the lists as they were), each track's
    /// applied pointer goes to the first node after the current tick, and [`Sequencer::refresh`]
    /// applies the restored tracks as the command's own last step does.
    pub fn swap_pool(&mut self, pool: &mut Pool, devices: &mut Devices) {
        std::mem::swap(&mut self.pool, pool);
        let t = self.tick_pos();
        for tr in &mut self.tracks {
            tr.next = tr.state_at(&self.pool, t).1;
        }
        self.revision += 1;
        self.refresh(devices);
    }

    /// The nodes this song's pool has free (the module doc).
    pub fn free_nodes(&self) -> usize {
        self.pool.free()
    }

    /// `sub_1001F520` on every track, in track order.
    fn merge_all(&mut self, t: i32) {
        if (0..TRACKS).any(|i| !self.recorded(i).is_empty()) {
            self.merges += 1;
        }
        for tr in &mut self.tracks {
            tr.merge(&mut self.pool, &mut self.buf, t);
        }
        self.revision += 1;
    }

    /// The record hook `sub_10012BC0` → `sub_10021340`, which the panel's controller messages
    /// call after their store ([`crate::frame::Engine::device_param`] and the mixer's): only in
    /// song mode with record on (globals bytes 0 and 2, 0x10021352, 0x10021367), device `d`'s
    /// track records `ctrl` = `value` at the current tick ([`Track::record`]'s rounding and
    /// loop rule).
    ///
    /// Not modelled: the warning the DLL sends the host when its pool has 99, 50 or fewer than
    /// 10 nodes free (top-level msg 79, `sub_1001D440`, 0x10021378), and a controller above
    /// 127, which no panel message records (the DLL's hold, quantisation and merge arrays are
    /// 128 bytes; such a controller reads and writes past them).
    pub fn record_hook(&mut self, d: Device, ctrl: u8, value: u8) {
        if !self.song_mode() || self.globals.byte2 == 0 || ctrl > 127 {
            return;
        }
        let now = self.tick_pos();
        let playing = self.playing();
        let g = &self.globals;
        let wrap = (g.loop_on != 0).then_some((g.loop_start as i32, g.loop_end as i32));
        // 0x100213BF: the track whose device id (+0x104) is d's
        if let Some(tr) = self.tracks.iter_mut().find(|t| t.device == d) {
            tr.record(&mut self.pool, now, playing, wrap, ctrl, value);
        }
    }

    /// Song msg 35 `{3,v}` (`sub_10021110`).
    pub fn set_loop_on(&mut self, v: u8) {
        self.globals.loop_on = v;
    }

    /// Song msg 35 `{4,v}` (`sub_1001FCD0`), BPM*1000.
    pub fn set_tempo(&mut self, v: u32) {
        self.globals.tempo = v;
    }

    /// Song msg 35 `{12,v}` (`sub_10021130`).
    pub fn set_loop_start(&mut self, v: u32) {
        self.globals.loop_start = v;
    }

    /// Song msg 35 `{16,v}` (`sub_10021150`).
    pub fn set_loop_end(&mut self, v: u32) {
        self.globals.loop_end = v;
    }

    /// Song msg 35 `{20,v}` (`sub_10021170`).
    pub fn set_shuffle(&mut self, v: u8) {
        self.globals.shuffle = v;
    }

    /// Song msg 37 (`sub_10021010`): each event of a batch is inserted into its track, from the
    /// node pool. A batch of more events than the pool has free nodes (`last - first` at
    /// 0x10021042 against `sub_1000DEE0`, `cmp ebx, eax; ja` at 0x1002105B, unsigned) is
    /// refused whole: nothing is inserted and the reply is `['K', 22]` (`sub_1001DE00`,
    /// 0x100210ED) instead of `['I']` (`sub_1001DF80`, 0x100210CA). Returns whether the batch
    /// was taken. A taken batch can still lose its last event: with as many events as free
    /// nodes, the last insert finds one node free and stores nothing (`sub_10014434`).
    #[must_use]
    pub fn add_events(&mut self, track: usize, events: &[song::TrackEvent]) -> bool {
        if events.len() > self.pool.free() {
            return false;
        }
        let tr = &mut self.tracks[track];
        for e in events {
            tr.insert_pooled(&mut self.pool, e.tick as i32, e.ctrl, e.value);
        }
        self.revision += 1;
        true
    }
}

/// The unit tests' track: one track with a pool of its own.
#[cfg(test)]
pub(crate) struct Solo {
    pub track: Track,
    pub pool: Pool,
    buf: MergeBuf,
}

#[cfg(test)]
impl std::ops::Deref for Solo {
    type Target = Track;
    fn deref(&self) -> &Track {
        &self.track
    }
}

#[cfg(test)]
impl std::ops::DerefMut for Solo {
    fn deref_mut(&mut self) -> &mut Track {
        &mut self.track
    }
}

#[cfg(test)]
impl Solo {
    pub fn new(device: Device) -> Self {
        Solo {
            track: Track::new(device),
            pool: Pool::new(),
            buf: MergeBuf::default(),
        }
    }

    pub fn track_mut(&mut self) -> TrackMut<'_> {
        TrackMut {
            track: &mut self.track,
            pool: &mut self.pool,
        }
    }

    pub fn insert(&mut self, tick: i32, ctrl: u8, value: u8) -> bool {
        self.track.insert(&mut self.pool, tick, ctrl, value)
    }

    pub fn events(&self) -> &[Event] {
        self.track.events(&self.pool)
    }

    pub fn recorded(&self) -> &[Event] {
        self.track.recorded(&self.pool)
    }

    pub fn chase(&mut self, p: i32) -> [u8; 128] {
        self.track.chase(&self.pool, p)
    }

    pub fn advance(&mut self, t: i32, apply: impl FnMut(Event)) {
        self.track.advance(&self.pool, t, apply)
    }

    pub fn prune(&mut self) {
        self.track.prune(&mut self.pool)
    }

    pub fn record(
        &mut self,
        now: i32,
        playing: bool,
        wrap: Option<(i32, i32)>,
        ctrl: u8,
        value: u8,
    ) {
        self.track
            .record(&mut self.pool, now, playing, wrap, ctrl, value)
    }

    pub fn merge(&mut self, t: i32) {
        self.track.merge(&mut self.pool, &mut self.buf, t)
    }

    pub fn free(&self) -> usize {
        self.pool.free()
    }
}
