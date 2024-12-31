pub struct MidiMessage {
    message_type: MessageType,
    channel: u8,
    data1: u8,
    data2: u8,
}

#[derive(Debug)]
pub enum MessageType {
    NoteOff,
    NoteOn,
    PolyphonicKeyPressure,
    ControlChange,
    ProgramChange,
    ChannelPressure,
    PitchBendChange,
    SystemMessage,
    Unknown,
}

impl MidiMessage {
    pub fn from_bytes(data: &[u8]) -> Self {
        if data.is_empty() {
            return Self::empty();
        }

        let status = data[0];
        let message_type = match status >> 4 {
            0x8 => MessageType::NoteOff,
            0x9 => MessageType::NoteOn,
            0xA => MessageType::PolyphonicKeyPressure,
            0xB => MessageType::ControlChange,
            0xC => MessageType::ProgramChange,
            0xD => MessageType::ChannelPressure,
            0xE => MessageType::PitchBendChange,
            0xF => MessageType::SystemMessage,
            _ => MessageType::Unknown,
        };

        let channel = status & 0x0F;
        let data1 = data.get(1).copied().unwrap_or(0);
        let data2 = data.get(2).copied().unwrap_or(0);

        Self {
            message_type,
            channel,
            data1,
            data2,
        }
    }

    fn empty() -> Self {
        Self {
            message_type: MessageType::Unknown,
            channel: 0,
            data1: 0,
            data2: 0,
        }
    }

    pub fn to_string(&self) -> String {
        match self.message_type {
            MessageType::NoteOn => format!(
                "Note On - Channel: {} Note: {} Velocity: {}",
                self.channel + 1,
                self.data1,
                self.data2
            ),
            MessageType::NoteOff => format!(
                "Note Off - Channel: {} Note: {} Velocity: {}",
                self.channel + 1,
                self.data1,
                self.data2
            ),
            MessageType::ControlChange => format!(
                "Control Change - Channel: {} Controller: {} Value: {}",
                self.channel + 1,
                self.data1,
                self.data2
            ),
            MessageType::ProgramChange => format!(
                "Program Change - Channel: {} Program: {}",
                self.channel + 1,
                self.data1
            ),
            _ => format!(
                "Other Message - Type: {:?} Channel: {} Data: [{}, {}]",
                self.message_type,
                self.channel + 1,
                self.data1,
                self.data2
            ),
        }
    }
} 