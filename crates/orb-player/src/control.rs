//! The output stream's owner (task-191): opens a [`Request`], reopens it when the choice
//! changes, when the system default device changes (a stream asked for "System default" follows
//! it) or when the device disappears, and reports what happened.
//!
//! The engine ([`Audio`]) lives in the stream's callback and is handed back when the stream is
//! stopped ([`Stream::stop`]), so a reopen costs a short gap and the song keeps playing: the
//! engine's state is not touched. There is no automatic buffer growth: the size asked for is the
//! size opened, and an underrun is counted and shown ([`InUse::underruns`]), not acted on.
//!
//! The rules of what falls back to what are in [`Controller::open_chain`]:
//! 1. a named device that is missing opens the system default instead (a note);
//! 2. a request that fails to open opens the default request (an error);
//! 3. the default request that fails leaves the panel without sound (an error), and the controller
//!    tries again on the system default device every two seconds, silently.

use std::time::{Duration, Instant};

use crate::Audio;
use crate::devices::{self, Request};
use crate::output::{self, Error, Health, InUse, Stream};

/// How often the system default device is looked at.
const WATCH_EVERY: Duration = Duration::from_secs(1);
/// How often a controller without a stream tries to open one again.
const RETRY_EVERY: Duration = Duration::from_secs(2);

/// Something the owner of the window must tell the user.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// A change that is not an error: the status line.
    Note(String),
    /// A failure: the error box.
    Error(String),
}

/// Owns the stream and the engine.
pub struct Controller {
    stream: Option<Stream>,
    /// The engine while no stream runs it.
    audio: Option<Audio>,
    /// The request the stream runs (or ran last).
    request: Request,
    /// When the default device or a retry was looked at last.
    checked: Instant,
    /// A stall was reported already: one error box per stream.
    stall_reported: bool,
    /// A test seam: stands for the drivers and the opener (see [`Controller::for_test_opener`]).
    seam: Option<Seam>,
}

/// A fake of `output::open`: decides a request, makes no stream.
type Opener = Box<dyn FnMut(&Request) -> Result<(), Error> + Send>;

/// What a test puts in place of the machine's ASIO drivers and of `output::open`.
struct Seam {
    /// The ASIO devices in the driver order, or the host's error.
    list: Result<Vec<devices::DeviceRef>, String>,
    /// Opens a request (no stream is made); the requests it was given are kept.
    open: Opener,
}

impl Controller {
    /// Opens `request` for `audio`, with the fallbacks of [`Controller::open_chain`].
    pub fn start(audio: Audio, request: Request) -> (Self, Vec<Event>) {
        let mut c = Controller {
            stream: None,
            audio: Some(audio),
            request: request.clone(),
            checked: Instant::now(),
            stall_reported: false,
            seam: None,
        };
        let events = c.open_chain(request);
        (c, events)
    }

    /// A controller without a stream or an engine that holds `request`: a test of its owner
    /// simulates a fallback with it, without a device.
    #[doc(hidden)]
    pub fn for_test(request: Request) -> Self {
        Controller {
            stream: None,
            audio: None,
            request,
            checked: Instant::now(),
            stall_reported: false,
            seam: None,
        }
    }

    /// A controller whose opener and ASIO drivers are fakes: `open_chain` and `apply` run their
    /// real rules, `open` decides each request (no stream is made, [`Controller::is_open`] stays
    /// false), and `list` is what the ASIO host lists or its error. For tests; no device needed.
    #[doc(hidden)]
    pub fn for_test_opener(
        request: Request,
        list: Result<Vec<devices::DeviceRef>, String>,
        open: impl FnMut(&Request) -> Result<(), Error> + Send + 'static,
    ) -> Self {
        let mut c = Controller::for_test(request);
        c.seam = Some(Seam {
            list,
            open: Box::new(open),
        });
        c
    }

    /// Whether this controller's drivers are the fakes of [`Controller::for_test_opener`].
    #[doc(hidden)]
    pub fn has_fake_drivers(&self) -> bool {
        self.seam.is_some()
    }

    /// The request in use (after the fallbacks), or the last one that was.
    pub fn request(&self) -> &Request {
        &self.request
    }

    /// Whether a stream is running.
    pub fn is_open(&self) -> bool {
        self.stream.is_some()
    }

    /// The running stream's health.
    pub fn health(&self) -> Option<Health> {
        self.stream.as_ref().map(Stream::health)
    }

    /// What the running stream uses.
    pub fn in_use(&self) -> Option<InUse> {
        self.stream.as_ref().map(Stream::in_use)
    }

    /// The running stream, for the start line.
    pub fn stream(&self) -> Option<&Stream> {
        self.stream.as_ref()
    }

    /// Runs the stream with `request` instead: a no-op when it is the one running.
    pub fn apply(&mut self, request: Request) -> Vec<Event> {
        if self.stream.is_some() && request == self.request {
            return Vec::new();
        }
        self.release();
        self.open_chain(request)
    }

    /// Looks at the stream: a lost device is replaced by the system default, a changed system
    /// default is followed, a stall is reported once, and a controller without a stream tries
    /// again. Cheap enough to call on every frame.
    pub fn poll(&mut self) -> Vec<Event> {
        let Some(s) = &self.stream else {
            return self.retry();
        };
        if let Some(why) = s.failure() {
            if s.health().device_gone() {
                let note = format!("Audio device {} was lost", s.device);
                self.release();
                let mut ev = vec![Event::Note(note)];
                ev.extend(self.open_chain(self.request.on_system_default()));
                return ev;
            }
            if !self.stall_reported {
                self.stall_reported = true;
                return vec![Event::Error(format!("Audio output: {why}"))];
            }
            return Vec::new();
        }
        // ASIO has no system default to follow (looking would load every driver in turn)
        if self.request.device.is_none()
            && !devices::is_asio(self.request.driver.as_deref())
            && self.checked.elapsed() >= WATCH_EVERY
        {
            self.checked = Instant::now();
            let now = devices::default_device_id(self.request.driver.as_deref());
            if let Some(id) = now
                && id != s.device_id
            {
                self.release();
                let mut ev = Vec::new();
                ev.extend(self.open_chain(self.request.clone()));
                if let Some(s) = &self.stream {
                    ev.insert(
                        0,
                        Event::Note(format!("Audio output changed: {}", s.device)),
                    );
                }
                return ev;
            }
        }
        Vec::new()
    }

    /// Stops the stream and takes the engine back.
    fn release(&mut self) {
        if let Some(s) = self.stream.take() {
            self.audio = s.stop();
        }
    }

    /// Opens one request: the stream is `Some` after `Ok`; a refusal gives the engine back.
    fn try_open(&mut self, request: &Request) -> Result<(), Error> {
        if let Some(seam) = &mut self.seam {
            return (seam.open)(request);
        }
        let Some(audio) = self.audio.take() else {
            return Err(Error(
                "the audio engine is lost; restart the program".into(),
            ));
        };
        match output::open(audio, request) {
            Ok(s) => {
                self.stream = Some(s);
                self.stall_reported = false;
                self.checked = Instant::now();
                Ok(())
            }
            Err(r) => {
                self.audio = r.audio;
                Err(r.error)
            }
        }
    }

    /// Opens `request` with the fallbacks:
    /// 1. a named device that is not there opens on the system default (a note);
    /// 2. a request the device refuses opens the default request (an error that says so);
    /// 3. the default request refused leaves no stream (an error).
    pub fn open_chain(&mut self, request: Request) -> Vec<Event> {
        let mut events = Vec::new();
        let mut request = request;
        if let Some(d) = &request.device {
            let there = self.seam.is_some()
                || devices::host(request.driver.as_deref())
                    .ok()
                    .and_then(|h| devices::find_device(&h, &d.id))
                    .is_some();
            if !there {
                events.push(Event::Note(format!(
                    "Audio device {} is not available; using System default",
                    d.name
                )));
                request = request.on_system_default();
            }
        }
        let asio = devices::is_asio(request.driver.as_deref());
        // ASIO has no system default device: with none named, the drivers are tried in turn
        // (each refusing one is dropped before the next loads: asio-sys holds one at a time)
        let mut refused: Vec<(String, String)> = Vec::new();
        self.request = request.clone();
        let first = if asio && request.device.is_none() {
            let listed: Result<Vec<devices::DeviceRef>, String> = match &self.seam {
                Some(s) => s.list.clone(),
                None => devices::list_devices(request.driver.as_deref())
                    .map(|v| {
                        v.into_iter()
                            .map(|d| devices::DeviceRef {
                                id: d.id,
                                name: d.name,
                            })
                            .collect()
                    })
                    .map_err(|e| e.0),
            };
            let list = listed.clone().unwrap_or_default();
            let (won, skipped) = try_in_turn(&list, |d| {
                let named = Request {
                    device: Some(d.clone()),
                    ..request.clone()
                };
                self.try_open(&named).map(|()| named)
            });
            refused = skipped;
            if let Some(named) = won {
                if !refused.is_empty() {
                    let used = named.device.as_ref().map(|d| d.name.clone());
                    events.push(Event::Error(refusal_text(
                        &refused,
                        &Instead::Driver(used.unwrap_or_default()),
                    )));
                    // the window shows the driver in use (task-245's way)
                    self.request = named;
                }
                self.checked = Instant::now();
                return events;
            }
            Error(match listed {
                Err(e) => e,
                Ok(_) if refused.is_empty() => "no ASIO driver is installed".into(),
                Ok(_) => "no ASIO driver opened".into(),
            })
        } else {
            let Err(e) = self.try_open(&request) else {
                return events;
            };
            if asio && let Some(d) = &request.device {
                refused.push((d.name.clone(), e.0.clone()));
            }
            e
        };
        if request.is_default() {
            events.push(Event::Error(format!(
                "Audio output: {first}; the panel runs without sound"
            )));
            self.checked = Instant::now();
            return events;
        }
        self.request = Request::default();
        let result = self.try_open(&Request::default());
        let said = if refused.is_empty() {
            format!("Audio output: {first}; using the System default with the default settings")
        } else {
            refusal_text(&refused, &Instead::Default)
        };
        match result {
            Ok(()) => events.push(Event::Error(said)),
            Err(e) => events.push(Event::Error(format!(
                "{}; the default failed too ({e}); the panel runs without sound",
                said.trim_end_matches('.')
            ))),
        }
        self.checked = Instant::now();
        events
    }

    /// Without a stream: every two seconds, one silent try on the system default (the device
    /// that was gone may be back, or another one plugged in).
    fn retry(&mut self) -> Vec<Event> {
        if self.audio.is_none()
            || self.request.device.is_some()
            || self.checked.elapsed() < RETRY_EVERY
        {
            return Vec::new();
        }
        self.checked = Instant::now();
        if devices::default_device_id(self.request.driver.as_deref()).is_none() {
            return Vec::new();
        }
        let request = self.request.clone();
        match self.try_open(&request) {
            Ok(()) => match &self.stream {
                Some(s) => vec![Event::Note(format!("Audio output is back on {}", s.device))],
                None => Vec::new(),
            },
            Err(_) => Vec::new(),
        }
    }
}

/// What the program did instead of the ASIO drivers that did not open.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instead {
    /// It used this other ASIO driver.
    Driver(String),
    /// It went back to the platform's default driver, System default, default settings.
    Default,
}

/// Tries each of `list` in order with `open` and returns the first that opens (`Some`) and the
/// ones that refused before it (name, reason). A refusing driver is skipped, never retried;
/// `open` drops what it loaded when it refuses, so one driver is loaded at a time (asio-sys's
/// rule). No device is needed: a test passes a fake opener.
pub fn try_in_turn<T>(
    list: &[devices::DeviceRef],
    mut open: impl FnMut(&devices::DeviceRef) -> Result<T, Error>,
) -> (Option<T>, Vec<(String, String)>) {
    let mut refused = Vec::new();
    for d in list {
        match open(d) {
            Ok(t) => return (Some(t), refused),
            Err(e) => refused.push((d.name.clone(), e.0)),
        }
    }
    (None, refused)
}

/// A driver's reason in plain words: the stream step's prefix dropped, and ASIO's "no clock" (a
/// driver whose device is not connected) with the likely cause.
pub fn plain_reason(reason: &str) -> String {
    let r = reason.strip_prefix("the output stream: ").unwrap_or(reason);
    if r.contains("sample clock or rate cannot be determined") {
        format!("{r}; is its device connected, or the program it needs running?")
    } else {
        r.to_string()
    }
}

/// The message about ASIO drivers that did not open: each one's name and reason, then what the
/// program did instead.
pub fn refusal_text(refused: &[(String, String)], instead: &Instead) -> String {
    let did = refused
        .iter()
        .map(|(n, r)| format!("{n} did not open ({})", plain_reason(r)))
        .collect::<Vec<_>>()
        .join("; ");
    let then = match instead {
        Instead::Driver(n) => format!("Using {n}."),
        Instead::Default => format!(
            "Back on {}, System default.",
            cpal::default_host().id().name()
        ),
    };
    format!("Audio output: {did}. {then}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(names: &[&str]) -> Vec<devices::DeviceRef> {
        names
            .iter()
            .map(|n| devices::DeviceRef {
                id: format!("asio:{n}"),
                name: n.to_string(),
            })
            .collect()
    }

    const NO_CLOCK: &str = "the output stream: sample clock or rate cannot be determined or \
                            is not present";

    fn fake(
        works: &'static [&'static str],
    ) -> impl FnMut(&devices::DeviceRef) -> Result<String, Error> {
        move |d| {
            if works.contains(&d.name.as_str()) {
                Ok(d.name.clone())
            } else {
                Err(Error(NO_CLOCK.into()))
            }
        }
    }

    #[test]
    fn the_first_driver_that_opens_is_used_and_a_refusing_one_skipped() {
        let l = list(&[
            "MiniFuse ASIO Driver",
            "Realtek ASIO",
            "ReaRoute ASIO (x64)",
        ]);
        let mut tried = Vec::new();
        let mut open = fake(&["Realtek ASIO", "ReaRoute ASIO (x64)"]);
        let (won, refused) = try_in_turn(&l, |d| {
            tried.push(d.name.clone());
            open(d)
        });
        assert_eq!(won.as_deref(), Some("Realtek ASIO"));
        assert_eq!(tried, ["MiniFuse ASIO Driver", "Realtek ASIO"]);
        assert_eq!(refused.len(), 1);
        assert_eq!(refused[0].0, "MiniFuse ASIO Driver");
    }

    /// A controller on ASIO at System default whose opener accepts only `works`; the shared
    /// channel holds every request the opener saw.
    fn chain(
        names: &[&str],
        works: &'static [&'static str],
        default_works: bool,
    ) -> (Controller, rtrb::Consumer<Request>) {
        let (mut seen, log) = rtrb::RingBuffer::new(16);
        let c =
            Controller::for_test_opener(Request::default(), Ok(list(names)), move |r: &Request| {
                let _ = seen.push(r.clone());
                let ok = match &r.device {
                    Some(d) => works.contains(&d.name.as_str()),
                    None => r.driver.is_none() && default_works,
                };
                if ok {
                    Ok(())
                } else {
                    Err(Error(NO_CLOCK.into()))
                }
            });
        (c, log)
    }

    fn asio() -> Request {
        Request {
            driver: Some("ASIO".into()),
            ..Request::default()
        }
    }

    fn errors(events: &[Event]) -> Vec<&str> {
        events
            .iter()
            .filter_map(|e| match e {
                Event::Error(m) => Some(m.as_str()),
                Event::Note(_) => None,
            })
            .collect()
    }

    #[test]
    fn open_chain_skips_a_refusing_driver_and_runs_the_next() {
        let (mut c, log) = chain(&["MiniFuse", "Realtek ASIO"], &["Realtek ASIO"], true);
        let ev = c.open_chain(asio());
        assert_eq!(errors(&ev).len(), 1, "{ev:?}");
        assert_eq!(
            errors(&ev)[0],
            refusal_text(
                &[("MiniFuse".into(), NO_CLOCK.into())],
                &Instead::Driver("Realtek ASIO".into())
            )
        );
        // the request is now the driver in use, on ASIO
        assert_eq!(c.request().driver.as_deref(), Some("ASIO"));
        assert_eq!(c.request().device.as_ref().unwrap().name, "Realtek ASIO");
        assert_eq!(log.slots(), 2);
    }

    #[test]
    fn open_chain_with_the_first_driver_opening_says_nothing_and_keeps_the_request() {
        let (mut c, _) = chain(&["A", "B"], &["A", "B"], true);
        let ev = c.open_chain(asio());
        assert!(ev.is_empty(), "{ev:?}");
        assert_eq!(c.request(), &asio());
    }

    #[test]
    fn open_chain_with_every_driver_refused_falls_back_to_the_default() {
        let (mut c, log) = chain(&["A", "B"], &[], true);
        let ev = c.open_chain(asio());
        assert_eq!(errors(&ev).len(), 1, "{ev:?}");
        assert_eq!(
            errors(&ev)[0],
            refusal_text(
                &[("A".into(), NO_CLOCK.into()), ("B".into(), NO_CLOCK.into())],
                &Instead::Default
            )
        );
        assert_eq!(c.request(), &Request::default());
        let mut log = log;
        let mut last = None;
        while let Ok(r) = log.pop() {
            last = Some(r);
        }
        assert_eq!(last.unwrap(), Request::default());
    }

    #[test]
    fn open_chain_with_the_default_refused_too_says_so_once() {
        let (mut c, _) = chain(&["A"], &[], false);
        let ev = c.open_chain(asio());
        assert_eq!(errors(&ev).len(), 1, "{ev:?}");
        let e = errors(&ev)[0];
        assert!(e.starts_with("Audio output: A did not open"), "{e}");
        assert!(e.contains("the default failed too"), "{e}");
        assert!(e.ends_with("the panel runs without sound"), "{e}");
    }

    #[test]
    fn a_named_asio_device_that_refuses_gets_the_same_text() {
        let (mut c, _) = chain(&["A", "B"], &[], true);
        let mut req = asio();
        req.device = Some(list(&["B"]).remove(0));
        let ev = c.open_chain(req);
        assert_eq!(errors(&ev).len(), 1, "{ev:?}");
        assert_eq!(
            errors(&ev)[0],
            refusal_text(&[("B".into(), NO_CLOCK.into())], &Instead::Default)
        );
        assert_eq!(c.request(), &Request::default());
    }

    #[test]
    fn a_host_error_is_reported_and_no_driver_installed_is_said_plainly() {
        let mut c = Controller::for_test_opener(
            Request::default(),
            Err("the ASIO host: boom".into()),
            |_| Err(Error("no".into())),
        );
        let ev = c.open_chain(asio());
        assert!(errors(&ev)[0].contains("the ASIO host: boom"), "{ev:?}");
        let mut c = Controller::for_test_opener(Request::default(), Ok(vec![]), |_| {
            Err(Error("no".into()))
        });
        let ev = c.open_chain(asio());
        assert!(
            errors(&ev)[0].contains("no ASIO driver is installed"),
            "{ev:?}"
        );
    }

    #[test]
    fn the_first_driver_when_it_opens_skips_nothing() {
        let (won, refused) = try_in_turn(&list(&["A", "B"]), fake(&["A", "B"]));
        assert_eq!(won.as_deref(), Some("A"));
        assert!(refused.is_empty());
    }

    #[test]
    fn when_none_opens_all_are_refused_in_order() {
        let (won, refused) = try_in_turn(&list(&["A", "B"]), fake(&[]));
        assert!(won.is_none());
        let names: Vec<_> = refused.iter().map(|r| r.0.as_str()).collect();
        assert_eq!(names, ["A", "B"]);
    }

    #[test]
    fn the_message_names_the_driver_the_reason_and_what_was_done() {
        let refused = vec![("MiniFuse ASIO Driver".to_string(), NO_CLOCK.to_string())];
        assert_eq!(
            refusal_text(&refused, &Instead::Driver("Realtek ASIO".into())),
            "Audio output: MiniFuse ASIO Driver did not open (sample clock or rate cannot be \
             determined or is not present; is its device connected, or the program it needs \
             running?). Using Realtek ASIO."
        );
        let two = vec![
            ("A".to_string(), "busy".to_string()),
            ("B".to_string(), "the output stream: no".to_string()),
        ];
        assert_eq!(
            refusal_text(&two, &Instead::Default),
            format!(
                "Audio output: A did not open (busy); B did not open (no). Back on {}, \
                 System default.",
                cpal::default_host().id().name()
            )
        );
    }
}
