// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Mateus Rocha de Paula and the OxiGuitar contributors

//! The `audio-device` service (plugin interface 1.3) on one of cpal's
//! hosts: what OxiGuitar's four audio-device plugins (PipeWire, ALSA,
//! WASAPI, CoreAudio) share. Each plugin names its host and adds nothing
//! else.
//!
//! Streams live on a thread of their own (a cpal stream need not move
//! between threads), which sleeps until asked to open or close one. A
//! stream's callback hands the core its buffer (`AudioOut::render`) with
//! the delay to the speaker that cpal reports for it; the callback itself
//! allocates nothing and waits for nothing.
//!
//! The rate: 48 kHz when the device takes it (NAM's), else the device's
//! own; f32 frames when the device takes them, else 16- or 32-bit integers
//! converted through a buffer made when the stream opens.
//!
//! Decisions and findings, for whoever changes this next:
//!
//! - This crate lived in OxiGuitar's repository until the core's v0.21.6 and
//!   moved here because no core crate used it: being in the core's
//!   workspace made every core build compile cpal and need ALSA's headers.
//!   It depends on the SDK at the same tag as the four plugins (v0.11.0),
//!   because its `Service` and `Host` must be the very types theirs are:
//!   move the tag here and in the four plugins together.
//! - cpal is used as released, with `default-features = false`; each plugin
//!   turns on its own host's feature (`pipewire`) on its own `cpal`
//!   dependency and cargo unifies the two. Upstream (cpal 0.18): on Linux
//!   cpal always builds its ALSA host too (Mateus, 25 Oct 2026: accepted).
//! - The device's own delay (`OutputCallbackInfo::timestamp`, playback
//!   minus callback) is what the core uses to show the cursor where the
//!   sound is heard. Upstream (cpal 0.18): a host that reports no delay
//!   gives 0, and the cursor then runs a buffer early. On update: check how
//!   each host fills the timestamp.
//! - Integer devices. Upstream (cpal 0.18): cpal offers no conversion of
//!   its own on the output side, so f32 samples are clamped and scaled here
//!   (`to_i16`, `to_i32`). On update: if cpal converts for us, these and
//!   `CONVERT` can go.
//!
//! Notes marked `Upstream (CRATE VERSION)` follow OxiGuitar's convention
//! (its CONTRIBUTING.md, "Comments in the code"): what a library does or
//! lacks, what we do about it, and what to check when it moves.
//! - Formats other than f32, i16 and i32 (u8, i8, i64, f64, u16, ...) are
//!   refused: no device we have seen offers only those.

use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{HostId, SampleFormat, StreamConfig};
use oxiguitar_plugin::api::audio_device::{CLOSE, DEVICES, OPEN, Open, Opened};
use oxiguitar_plugin::api::{decode, encode};
use oxiguitar_plugin::{Availability, Error, ErrorKind, Host, Level, Service};

/// The most frames a callback converts at once from f32 (for devices that
/// take integers): larger buffers are converted in pieces.
const CONVERT: usize = 8192;

/// An audio-device service on cpal's host `id`.
pub struct CpalDevice {
    id: HostId,
    name: &'static str,
    host: Host,
    jobs: Mutex<Option<Sender<Job>>>,
}

/// What the streams' thread is asked.
enum Job {
    Open(Open, Sender<Result<Opened, Error>>),
    Close(u64, Sender<()>),
}

impl std::fmt::Debug for CpalDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CpalDevice")
            .field("host", &self.name)
            .finish()
    }
}

impl CpalDevice {
    /// The service on cpal's host `id`, called `name` in messages.
    pub fn new(host: Host, id: HostId, name: &'static str) -> CpalDevice {
        CpalDevice {
            id,
            name,
            host,
            jobs: Mutex::new(None),
        }
    }

    fn cpal_host(&self) -> Result<cpal::Host, Error> {
        cpal::host_from_id(self.id).map_err(|e| {
            Error::new(
                ErrorKind::Unsupported,
                format!("{} isn't available here ({e})", self.name),
            )
        })
    }

    /// The streams' thread, started on first use.
    fn jobs(&self) -> Result<Sender<Job>, Error> {
        let mut jobs = self
            .jobs
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(j) = jobs.as_ref() {
            return Ok(j.clone());
        }
        let (tx, rx) = channel();
        let host = self.host.clone();
        let id = self.id;
        let name = self.name;
        std::thread::Builder::new()
            .name(format!("oxiguitar-{}", name.to_lowercase()))
            .spawn(move || keep_streams(&rx, &host, id, name))
            .map_err(|e| {
                Error::new(ErrorKind::Failed, format!("no thread for the streams: {e}"))
            })?;
        *jobs = Some(tx.clone());
        Ok(tx)
    }
}

impl Service for CpalDevice {
    fn probe(&self) -> Availability {
        match self.cpal_host() {
            Err(e) => Availability::Unavailable(e.message),
            Ok(host) => match host.default_output_device() {
                Some(_) => Availability::Available,
                None => Availability::Unavailable(format!("{} has no output", self.name)),
            },
        }
    }

    fn call(&self, method: &str, input: &[u8]) -> Result<Vec<u8>, Error> {
        let invalid = |e: oxiguitar_plugin::api::DecodeError| {
            Error::new(ErrorKind::Invalid, format!("the input can't be read ({e})"))
        };
        match method {
            OPEN => {
                let open: Open = decode(input).map_err(invalid)?;
                let (tx, rx) = channel();
                self.jobs()?.send(Job::Open(open, tx)).map_err(|_| gone())?;
                let opened = rx.recv().map_err(|_| gone())??;
                encode(&opened).map_err(unencoded)
            }
            CLOSE => {
                let stream: u64 = decode(input).map_err(invalid)?;
                let (tx, rx) = channel();
                self.jobs()?
                    .send(Job::Close(stream, tx))
                    .map_err(|_| gone())?;
                rx.recv().map_err(|_| gone())?;
                Ok(Vec::new())
            }
            DEVICES => {
                let host = self.cpal_host()?;
                let names: Vec<String> = host
                    .output_devices()
                    .map(|d| d.filter_map(|d| device_name(&d)).collect())
                    .unwrap_or_default();
                encode(&names).map_err(unencoded)
            }
            _ => Err(Error::not_found(format!("no method {method:?}"))),
        }
    }
}

/// An answer that couldn't be encoded.
fn unencoded(e: oxiguitar_plugin::api::DecodeError) -> Error {
    Error::new(ErrorKind::Failed, e.to_string())
}

fn gone() -> Error {
    Error::new(ErrorKind::Failed, "the audio streams' thread has ended")
}

fn device_name(d: &cpal::Device) -> Option<String> {
    d.description().ok().map(|d| d.name().to_owned())
}

/// The streams' thread: sleeps until asked, opens and closes streams.
fn keep_streams(jobs: &Receiver<Job>, host: &Host, id: HostId, name: &'static str) {
    let mut streams: Vec<(u64, cpal::Stream)> = Vec::new();
    while let Ok(job) = jobs.recv() {
        match job {
            Job::Open(open, reply) => {
                let result = open_stream(host, id, name, &open).map(|(stream, opened)| {
                    streams.push((open.stream, stream));
                    opened
                });
                let _ = reply.send(result);
            }
            Job::Close(n, reply) => {
                // Dropping a cpal stream stops it: its callback isn't called
                // again once this returns.
                streams.retain(|(s, _)| *s != n);
                let _ = reply.send(());
            }
        }
    }
}

/// A sample for a 16-bit device: clamped to full scale (a louder sample
/// would wrap round and click) and scaled to `i16::MAX`, so -1.0 is
/// -32767, not `i16::MIN`.
fn to_i16(v: f32) -> i16 {
    (v.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16
}

/// A sample for a 32-bit device, as `to_i16`; through f64, because f32 has
/// too few digits for `i32::MAX`.
fn to_i32(v: f32) -> i32 {
    (f64::from(v.clamp(-1.0, 1.0)) * f64::from(i32::MAX)) as i32
}

fn failed(what: &str, e: impl std::fmt::Display) -> Error {
    Error::new(ErrorKind::Failed, format!("{what} ({e})"))
}

fn open_stream(
    host: &Host,
    id: HostId,
    name: &'static str,
    open: &Open,
) -> Result<(cpal::Stream, Opened), Error> {
    let out = host
        .audio_out()
        .ok_or_else(|| Error::new(ErrorKind::Unsupported, "this OxiGuitar can't play sound"))?;
    let cpal_host =
        cpal::host_from_id(id).map_err(|e| failed(&format!("{name} isn't available"), e))?;
    let device = if open.device.is_empty() {
        // The system's default output, or the first that can be opened
        // (a machine whose default names no card).
        cpal_host
            .default_output_device()
            .filter(|d| d.default_output_config().is_ok())
            .or_else(|| {
                cpal_host
                    .output_devices()
                    .ok()?
                    .find(|d| d.default_output_config().is_ok())
            })
    } else {
        cpal_host
            .output_devices()
            .ok()
            .and_then(|mut d| d.find(|d| device_name(d).as_deref() == Some(open.device.as_str())))
    }
    .ok_or_else(|| Error::new(ErrorKind::NotFound, format!("{name} has no such output")))?;
    let default = device
        .default_output_config()
        .map_err(|e| failed("the output gives no configuration", e))?;
    // The rate asked for when the device takes it, in f32 if it can; else
    // the device's own.
    let wanted = device.supported_output_configs().ok().and_then(|configs| {
        let fits: Vec<_> = configs
            .filter(|c| c.min_sample_rate() <= open.rate && open.rate <= c.max_sample_rate())
            .collect();
        fits.iter()
            .find(|c| c.sample_format() == SampleFormat::F32 && c.channels() >= 2)
            .or_else(|| fits.iter().find(|c| c.sample_format() == SampleFormat::F32))
            .or_else(|| fits.first())
            .map(|c| c.with_sample_rate(open.rate))
    });
    let chosen = wanted.unwrap_or(default);
    let format = chosen.sample_format();
    let config: StreamConfig = chosen.config();
    let channels = usize::from(config.channels);
    let stream_no = open.stream;
    let errors = host.clone();
    let on_error = move |e: cpal::Error| {
        errors.notify(Level::Error, format!("{name}: the sound stopped ({e})."));
    };
    let delay = |info: &cpal::OutputCallbackInfo| {
        let ts = info.timestamp();
        u64::try_from(ts.playback.duration_since(ts.callback).as_nanos()).unwrap_or(u64::MAX)
    };
    let stream = match format {
        SampleFormat::F32 => device.build_output_stream(
            config,
            move |data: &mut [f32], info: &cpal::OutputCallbackInfo| {
                out.render(stream_no, data, channels, delay(info));
            },
            on_error,
            None,
        ),
        SampleFormat::I16 => {
            let mut scratch = vec![0.0f32; CONVERT * channels];
            device.build_output_stream(
                config,
                move |data: &mut [i16], info: &cpal::OutputCallbackInfo| {
                    let d = delay(info);
                    for chunk in data.chunks_mut(CONVERT * channels) {
                        let s = &mut scratch[..chunk.len()];
                        out.render(stream_no, s, channels, d);
                        for (o, v) in chunk.iter_mut().zip(s.iter()) {
                            *o = to_i16(*v);
                        }
                    }
                },
                on_error,
                None,
            )
        }
        SampleFormat::I32 => {
            let mut scratch = vec![0.0f32; CONVERT * channels];
            device.build_output_stream(
                config,
                move |data: &mut [i32], info: &cpal::OutputCallbackInfo| {
                    let d = delay(info);
                    for chunk in data.chunks_mut(CONVERT * channels) {
                        let s = &mut scratch[..chunk.len()];
                        out.render(stream_no, s, channels, d);
                        for (o, v) in chunk.iter_mut().zip(s.iter()) {
                            *o = to_i32(*v);
                        }
                    }
                },
                on_error,
                None,
            )
        }
        other => {
            return Err(Error::new(
                ErrorKind::Unsupported,
                format!("the output takes {other} samples, which OxiGuitar doesn't make"),
            ));
        }
    }
    .map_err(|e| failed("the output couldn't be opened", e))?;
    stream
        .play()
        .map_err(|e| failed("the output couldn't be started", e))?;
    let opened = Opened {
        rate: config.sample_rate,
        channels: u32::from(config.channels),
        frames: match config.buffer_size {
            cpal::BufferSize::Fixed(n) => n,
            cpal::BufferSize::Default => 0,
        },
        device: device_name(&device).unwrap_or_else(|| name.to_owned()),
    };
    Ok((stream, opened))
}

#[cfg(test)]
mod tests {
    use super::{CpalDevice, to_i16, to_i32};
    use oxiguitar_plugin::{Availability, ErrorKind, Host, Service};

    #[test]
    fn samples_for_integer_devices_are_clamped_and_scaled() {
        assert_eq!(to_i16(0.0), 0);
        assert_eq!(to_i16(1.0), i16::MAX);
        assert_eq!(to_i16(-1.0), -i16::MAX);
        assert_eq!(to_i16(2.5), i16::MAX, "louder than full scale");
        assert_eq!(to_i16(-7.0), -i16::MAX);
        assert_eq!(to_i16(0.5), 16383);
        assert_eq!(to_i32(0.0), 0);
        assert_eq!(to_i32(1.0), i32::MAX);
        assert_eq!(to_i32(-1.0), -i32::MAX);
        assert_eq!(to_i32(3.0), i32::MAX, "louder than full scale");
        assert_eq!(to_i32(0.5), 1_073_741_823);
    }

    /// A device on the host every system has (ALSA on Linux, WASAPI on
    /// Windows, Core Audio on macOS).
    fn device() -> CpalDevice {
        CpalDevice::new(Host::detached(), cpal::default_host().id(), "Test")
    }

    /// With or without a sound card (CI has none), probing answers, and
    /// says why when the device can't be used.
    #[test]
    fn probing_says_whether_there_is_an_output() {
        match device().probe() {
            Availability::Available => {}
            Availability::Unavailable(why) => assert!(!why.trim().is_empty()),
        }
    }

    /// The device list comes back as names, or as an error that says why.
    #[test]
    fn listing_the_outputs_answers_with_or_without_a_device() {
        use oxiguitar_plugin::api::audio_device::DEVICES;
        match device().call(DEVICES, &[]) {
            Ok(answer) => {
                let names: Vec<String> =
                    oxiguitar_plugin::api::decode(answer.as_slice()).expect("a list of names");
                assert!(names.iter().all(|n| !n.is_empty()));
            }
            Err(e) => assert!(!e.message.trim().is_empty()),
        }
    }

    /// A damaged request to open or close an output, and a method it
    /// doesn't have, are refused with an error rather than a crash.
    #[test]
    fn wrong_requests_are_refused() {
        use oxiguitar_plugin::api::audio_device::{CLOSE, OPEN};
        let d = device();
        assert_eq!(
            d.call(OPEN, b"not a request").unwrap_err().kind,
            ErrorKind::Invalid
        );
        assert_eq!(d.call(CLOSE, b"").unwrap_err().kind, ErrorKind::Invalid);
        assert_eq!(
            d.call("no-such-method", &[]).unwrap_err().kind,
            ErrorKind::NotFound
        );
    }

    /// Closing a stream that was never opened is not an error: the player
    /// closes what it may not have opened (an output that failed to start).
    #[test]
    fn closing_an_unknown_stream_is_harmless() {
        use oxiguitar_plugin::api::audio_device::CLOSE;
        let input = oxiguitar_plugin::api::encode(&42u64).expect("encodes");
        assert_eq!(
            device().call(CLOSE, &input).expect("closes"),
            Vec::<u8>::new()
        );
    }

    /// Opening an output in a plugin that runs outside OxiGuitar (no core
    /// to make the sound) is refused and says why, even with a sound card.
    #[test]
    fn opening_without_a_core_says_why() {
        use oxiguitar_plugin::api::audio_device::{OPEN, Open};
        let open = Open {
            stream: 1,
            rate: 48_000,
            frames: 0,
            device: String::new(),
        };
        let input = oxiguitar_plugin::api::encode(&open).expect("encodes");
        let e = device().call(OPEN, &input).unwrap_err();
        assert!(!e.message.trim().is_empty());
    }
}
