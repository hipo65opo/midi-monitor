use iced::{
    widget::{button, column, container, row, slider, text},
    Element, Length, Application, Command, Theme, Subscription, time,
};
use log;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::time::Duration;
use crate::midi::{MidiDevice, MidiMessage};

#[derive(Debug, Clone)]
pub enum Message {
    RefreshDevices,
    DeviceSelected(String),
    SliderChanged(f32),
    MidiMessageReceived(Vec<u8>),
    Tick,
}

pub struct MidiMonitor {
    midi_device: MidiDevice,
    midi_devices: Vec<String>,
    selected_device: Option<String>,
    messages: Vec<String>,
    message_receiver: Option<Receiver<Vec<u8>>>,
    message_sender: Option<Sender<Vec<u8>>>,
    slider_value: f32,
    last_sent_value: i32,
}

impl Application for MidiMonitor {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                midi_device: MidiDevice::new(),
                midi_devices: Vec::new(),
                selected_device: None,
                messages: Vec::new(),
                message_receiver: None,
                message_sender: None,
                slider_value: 0.0,
                last_sent_value: -1,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("MIDI Monitor")
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(10)).map(|_| Message::Tick)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::RefreshDevices => {
                if let Ok(devices) = MidiDevice::list_ports() {
                    self.midi_devices = devices;
                }
            }
            Message::DeviceSelected(device_name) => {
                let (sender, receiver) = mpsc::channel();
                if self.midi_device.connect(&device_name, sender.clone()).is_ok() {
                    self.selected_device = Some(device_name);
                    self.message_sender = Some(sender);
                    self.message_receiver = Some(receiver);
                }
            }
            Message::SliderChanged(value) => {
                self.slider_value = value;
                let current_value = value as i32;
                if current_value != self.last_sent_value {
                    if let Err(err) = self.midi_device.send_cc(0, 81, current_value as u8) {
                        log::error!("Failed to send MIDI CC: {}", err);
                    }
                    self.last_sent_value = current_value;
                }
            }
            Message::MidiMessageReceived(data) => {
                if let Some(message) = MidiMessage::from_bytes(&data) {
                    self.messages.push(message.to_string());
                    if self.messages.len() > 1000 {
                        self.messages.remove(0);
                    }

                    // CC#81を受信した場合、スライダーを更新
                    if message.is_control_change() && message.get_controller_number() == 81 {
                        self.slider_value = message.get_value() as f32;
                        self.last_sent_value = message.get_value() as i32;
                    }
                }
            }
            Message::Tick => {
                // MIDIメッセージの受信をチェック
                if let Some(receiver) = &self.message_receiver {
                    loop {
                        match receiver.try_recv() {
                            Ok(data) => {
                                return Command::perform(
                                    async move { data },
                                    Message::MidiMessageReceived,
                                );
                            }
                            Err(TryRecvError::Empty) => break,
                            Err(TryRecvError::Disconnected) => {
                                self.message_receiver = None;
                                break;
                            }
                        }
                    }
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        // メインコンテンツ（デバイス選択とメッセージ表示）
        let main_content = row![
            // 左側: コントロール部分
            column![
                text("MIDI Monitor").size(24),
                row![
                    text("MIDI Devices:"),
                    button("Refresh").on_press(Message::RefreshDevices)
                ],
                // デバイスリスト
                column(
                    self.midi_devices
                        .iter()
                        .map(|device| {
                            button(device.as_str())
                                .on_press(Message::DeviceSelected(device.clone()))
                                .into()
                        })
                        .collect()
                )
            ]
            .width(Length::FillPortion(2))
            .spacing(10),

            // 右側: メッセージ表示部分
            column(
                self.messages
                    .iter()
                    .map(|msg| text(msg).into())
                    .collect()
            )
            .width(Length::FillPortion(3))
        ];

        // スライダー部分（縦向き）
        let slider_container = column![
            text(format!("Value: {}", self.slider_value as i32)),
            container(
                slider(0.0..=127.0, self.slider_value, Message::SliderChanged)
                    .style(VerticalSliderStyle)
                    .width(40)
                    .height(200)
            )
            .height(200),
            text("CC#81")
        ]
        .spacing(10)
        .width(Length::Fixed(60.0))
        .align_items(iced::Alignment::Center);

        // 全体のレイアウト
        container(
            row![
                // スライダーを左側に配置
                container(slider_container).padding(10),
                // メインコンテンツを右側に配置
                container(main_content)
                    .width(Length::Fill)
                    .padding(20)
            ]
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

// スライダーのスタイル
impl From<VerticalSliderStyle> for iced::theme::Slider {
    fn from(_: VerticalSliderStyle) -> Self {
        iced::theme::Slider::Custom(Box::new(VerticalSliderStyle))
    }
}

pub struct VerticalSliderStyle;

impl slider::StyleSheet for VerticalSliderStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> slider::Appearance {
        slider::Appearance {
            rail: slider::Rail {
                colors: (
                    iced::Color::from_rgb(0.7, 0.7, 0.7),
                    iced::Color::from_rgb(0.5, 0.5, 0.5),
                ),
                width: 8.0,
                border_radius: 4.0.into(),
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Rectangle { 
                    width: 20.0, 
                    border_radius: 2.0.into() 
                },
                color: iced::Color::from_rgb(0.3, 0.3, 0.3),
                border_width: 1.0,
                border_color: iced::Color::from_rgb(0.2, 0.2, 0.2),
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> slider::Appearance {
        self.active(style)
    }

    fn dragging(&self, style: &Self::Style) -> slider::Appearance {
        slider::Appearance {
            handle: slider::Handle {
                color: iced::Color::from_rgb(0.2, 0.2, 0.2),
                ..self.active(style).handle
            },
            ..self.active(style)
        }
    }
} 