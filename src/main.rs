mod db;

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::{Context, Result};
use midir::{Ignore, MidiInput, MidiOutput};
use prost::bytes::Bytes;

fn main() -> Result<()> {
    let mut midi_in = MidiInput::new("midi-playback-in")?;
    midi_in.ignore(Ignore::None);

    let in_port = midi_in
        .ports()
        .into_iter()
        .nth(0)
        .context("no midi inputs detected")?;
    println!("using input at {}", midi_in.port_name(&in_port)?);

    let mut conn_in = midi_in
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

    std::thread::sleep(Duration::from_secs(10));

    let (_, phrase) = conn_in.close();

    let midi_out = MidiOutput::new("midi-playback-out")?;
    let out_port = midi_out
        .ports()
        .into_iter()
        .nth(0)
        .context("no midi outputs detected")?;
    println!("using output at {}", midi_out.port_name(&out_port)?);

    let mut conn_out = midi_out.connect(&out_port, "midir-forward")?;

    let mut timestamp = 0;
    for event in &phrase.events {
        std::thread::sleep(Duration::from_micros(event.timestamp - timestamp));
        conn_out.send(&event.data)?;
        timestamp = event.timestamp;
    }
    conn_out.close();

    Ok(())
}
