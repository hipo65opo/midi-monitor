use anyhow::Result;
use log;
use midir::{MidiInput, MidiInputPort, MidiInputConnection};
use std::sync::mpsc;

pub struct MidiDevice {
    connection: Option<MidiInputConnection<()>>,
}

impl MidiDevice {
    pub fn new() -> Self {
        Self {
            connection: None,
        }
    }

    pub fn list_ports() -> Result<Vec<String>> {
        let midi_in = MidiInput::new("MIDI Monitor")?;
        let ports = midi_in.ports();
        let mut port_names = Vec::new();

        for port in ports {
            if let Ok(name) = midi_in.port_name(&port) {
                port_names.push(name);
            }
        }

        Ok(port_names)
    }

    pub fn connect(&mut self, port_name: &str, sender: mpsc::Sender<Vec<u8>>) -> Result<()> {
        self.connection = None;

        let mut midi_in = MidiInput::new("MIDI Monitor")?;
        midi_in.ignore(midir::Ignore::None);

        let ports = midi_in.ports();
        let port = ports.into_iter()
            .find(|p| midi_in.port_name(p).map(|n| n == port_name).unwrap_or(false))
            .ok_or_else(|| anyhow::anyhow!("MIDI port not found"))?;

        log::info!("Connecting to MIDI port: {}", port_name);

        let connection = midi_in.connect(
            &port,
            "MIDI Monitor",
            move |_stamp, message, _| {
                let _ = sender.send(message.to_vec());
            },
            (),
        )?;

        self.connection = Some(connection);

        Ok(())
    }
} 