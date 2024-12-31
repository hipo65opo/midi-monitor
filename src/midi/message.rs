#[derive(Debug)]
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
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 3 {
            return None;
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
        let data1 = data[1];
        let data2 = data[2];

        Some(Self {
            message_type,
            channel,
            data1,
            data2,
        })
    }

    pub fn is_control_change(&self) -> bool {
        matches!(self.message_type, MessageType::ControlChange)
    }

    pub fn get_controller_number(&self) -> u8 {
        self.data1
    }

    pub fn get_value(&self) -> u8 {
        self.data2
    }

    pub fn to_string(&self) -> String {
        match self.message_type {
            MessageType::ControlChange => format!(
                "Control Change - Channel: {} Controller: {} Value: {}",
                self.channel + 1,
                self.data1,
                self.data2
            ),
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
            _ => format!(
                "MIDI Message - Type: {:?} Channel: {} Data: [{}, {}]",
                self.message_type,
                self.channel + 1,
                self.data1,
                self.data2
            ),
        }
    }
} 