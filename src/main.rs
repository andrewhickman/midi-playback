use std::{
    mem,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use anyhow::{Context, Result};
use midir::{Ignore, MidiInput, MidiOutput};
use nwg::NativeUi;

mod db;
mod ui;

#[derive(Clone, Default)]
struct PlaybackEngine {
    inner: Arc<Mutex<PlaybackEngineInner>>,
}

#[derive(Default)]
struct PlaybackEngineInner {
    recording: Option<db::Phrase>,
    state: PlaybackEngineState,
}

enum PlaybackEngineState {
    Idle,
    Recording {
        conn: midir::MidiInputConnection<db::Phrase>,
    },
    Playing {
        stop_handle: Arc<AtomicBool>,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    nwg::init().context("failed to initialize ui")?;
    nwg::Font::set_global_family("Segoe UI").context("failed to set default font")?;
    let _ui = ui::AppData::build_ui(Default::default()).expect("failed to build ui");
    nwg::dispatch_thread_events();
    Ok(())
}

impl PlaybackEngine {
    pub fn start_record(&self) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().unwrap();
        match inner.state {
            PlaybackEngineState::Idle => {
                let mut midi_in = MidiInput::new("midi-playback-in")?;
                midi_in.ignore(Ignore::None);

                let in_port = midi_in
                    .ports()
                    .into_iter()
                    .nth(0)
                    .context("no midi inputs detected")?;
                tracing::info!("using input at {}", midi_in.port_name(&in_port)?);

                let conn_in = midi_in
                    .connect(
                        &in_port,
                        "midi-playback-in",
                        move |timestamp, data, phrase| {
                            phrase.events.push(dbg!(db::PhraseEvent {
                                timestamp,
                                data: data.to_vec().into(),
                            }));
                        },
                        db::Phrase::default(),
                    )
                    .context("error connecting to input port")?;

                inner.state = PlaybackEngineState::Recording { conn: conn_in };
            }
            _ => (),
        }

        Ok(())
    }

    pub fn stop_record(&self) {
        let mut inner = self.inner.lock().unwrap();
        match &inner.state {
            PlaybackEngineState::Recording { .. } => (),
            _ => return,
        }

        match mem::replace(&mut inner.state, PlaybackEngineState::Idle) {
            PlaybackEngineState::Recording { conn } => {
                let (_, phrase) = conn.close();
                inner.recording = Some(phrase);
            }
            _ => unreachable!(),
        }
    }

    pub fn start_playback(&self) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        match inner.state {
            PlaybackEngineState::Idle => {
                let phrase = match &inner.recording {
                    Some(p) => p.clone(),
                    None => return Ok(()),
                };

                let midi_out = MidiOutput::new("midi-playback-out")?;
                let out_port = midi_out
                    .ports()
                    .into_iter()
                    .nth(0)
                    .context("no midi outputs detected")?;
                tracing::info!("using output at {}", midi_out.port_name(&out_port)?);

                let mut conn_out = midi_out.connect(&out_port, "midir-forward")?;

                let stop_handle = Arc::new(AtomicBool::new(false));
                let stop_handle_clone = stop_handle.clone();
                std::thread::spawn(move || {
                    let mut timestamp = 0;
                    for event in &phrase.events {
                        std::thread::sleep(Duration::from_micros(event.timestamp - timestamp));
                        if stop_handle_clone.load(Ordering::Relaxed) {
                            return;
                        }
                        conn_out.send(&event.data).ok();
                        timestamp = event.timestamp;
                    }
                    conn_out.close();
                });

                inner.state = PlaybackEngineState::Playing { stop_handle };
            }
            _ => (),
        }

        Ok(())
    }

    pub fn stop_playback(&self) {
        let inner = self.inner.lock().unwrap();
        match &inner.state {
            PlaybackEngineState::Playing { stop_handle } => {
                stop_handle.store(true, Ordering::Relaxed);
            }
            _ => (),
        }
    }
}

impl Default for PlaybackEngineState {
    fn default() -> Self {
        PlaybackEngineState::Idle
    }
}
