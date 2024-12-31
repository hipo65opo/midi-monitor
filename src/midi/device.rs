use anyhow::Result;
use log;
use midir::{MidiInput, MidiOutput, MidiInputConnection, MidiOutputConnection};
use std::sync::mpsc;

pub struct MidiDevice {
    input_connection: Option<MidiInputConnection<()>>,
    output_connection: Option<MidiOutputConnection>,
}

impl MidiDevice {
    pub fn new() -> Self {
        Self {
            input_connection: None,
            output_connection: None,
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
        self.input_connection = None;
        self.output_connection = None;

        let mut midi_in = MidiInput::new("MIDI Monitor Input")?;
        midi_in.ignore(midir::Ignore::None);

        let in_ports = midi_in.ports();
        let in_port = in_ports.into_iter()
            .find(|p| midi_in.port_name(p).map(|n| n == port_name).unwrap_or(false))
            .ok_or_else(|| anyhow::anyhow!("MIDI input port not found"))?;

        log::info!("Connecting to MIDI input port: {}", port_name);

        let input_connection = midi_in.connect(
            &in_port,
            "MIDI Monitor Input",
            move |_stamp, message, _| {
                let _ = sender.send(message.to_vec());
            },
            (),
        )?;

        let midi_out = MidiOutput::new("MIDI Monitor Output")?;
        let out_ports = midi_out.ports();
        let out_port = out_ports.into_iter()
            .find(|p| midi_out.port_name(p).map(|n| n == port_name).unwrap_or(false))
            .ok_or_else(|| anyhow::anyhow!("MIDI output port not found"))?;

        log::info!("Connecting to MIDI output port: {}", port_name);

        let output_connection = midi_out.connect(&out_port, "MIDI Monitor Output")?;

        self.input_connection = Some(input_connection);
        self.output_connection = Some(output_connection);

        Ok(())
    }

    pub fn send_cc(&mut self, channel: u8, controller: u8, value: u8) -> Result<()> {
        if let Some(conn) = &mut self.output_connection {
            let message = [0xB0 + (channel & 0x0F), controller, value];
            conn.send(&message)?;
            log::info!("Sent CC#{} value {} on channel {}", controller, value, channel + 1);
            Ok(())
        } else {
            Err(anyhow::anyhow!("No MIDI output connection available"))
        }
    }
} 