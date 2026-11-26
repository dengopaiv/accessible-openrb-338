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
        };
        let events = c.open_chain(request);
        (c, events)
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
            let there = devices::host(request.driver.as_deref())
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
        self.request = request.clone();
        let Err(first) = self.try_open(&request) else {
            return events;
        };
        if request.is_default() {
            events.push(Event::Error(format!(
                "Audio output: {first}; the panel runs without sound"
            )));
            self.checked = Instant::now();
            return events;
        }
        self.request = Request::default();
        match self.try_open(&Request::default()) {
            Ok(()) => events.push(Event::Error(format!(
                "Audio output: {first}; using the System default with the default settings"
            ))),
            Err(e) => events.push(Event::Error(format!(
                "Audio output: {first}; the default failed too ({e}); the panel runs without sound"
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
