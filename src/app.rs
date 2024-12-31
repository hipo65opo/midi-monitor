use eframe::egui;
use log;
use std::sync::mpsc::{self, Receiver, Sender};
use crate::midi::{MidiDevice, MidiMessage};

pub struct App {
    // MIDIデバイス管理
    midi_device: MidiDevice,
    midi_devices: Vec<String>,
    selected_device: Option<String>,
    
    // メッセージ処理
    messages: Vec<String>,
    message_receiver: Option<Receiver<Vec<u8>>>,
    message_sender: Option<Sender<Vec<u8>>>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        log::info!("Initializing application...");
        
        // デフォルトのスタイルを設定
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        cc.egui_ctx.set_style(style);

        Self {
            midi_device: MidiDevice::new(),
            midi_devices: Vec::new(),
            selected_device: None,
            messages: Vec::new(),
            message_receiver: None,
            message_sender: None,
        }
    }

    fn refresh_devices(&mut self) {
        match MidiDevice::list_ports() {
            Ok(devices) => {
                log::info!("Found {} MIDI devices", devices.len());
                self.midi_devices = devices;
            }
            Err(err) => {
                log::error!("Failed to list MIDI devices: {}", err);
            }
        }
    }

    fn connect_to_device(&mut self, device_name: String) {
        // チャネルを作成
        let (sender, receiver) = mpsc::channel();
        
        match self.midi_device.connect(&device_name, sender.clone()) {
            Ok(()) => {
                log::info!("Connected to MIDI device: {}", device_name);
                self.selected_device = Some(device_name);
                self.message_sender = Some(sender);
                self.message_receiver = Some(receiver);
            }
            Err(err) => {
                log::error!("Failed to connect to MIDI device: {}", err);
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // メッセージの受信と処理
        if let Some(receiver) = &self.message_receiver {
            while let Ok(data) = receiver.try_recv() {
                let message = MidiMessage::from_bytes(&data);
                self.messages.push(message.to_string());
                
                // メッセージ数が多すぎる場合は古いものを削除
                if self.messages.len() > 1000 {
                    self.messages.remove(0);
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("MIDI Monitor");
            
            // デバイス選択UI
            ui.group(|ui| {
                ui.label("MIDI Devices:");
                if ui.button("Refresh Devices").clicked() {
                    self.refresh_devices();
                }

                let devices = self.midi_devices.clone();
                for device in devices {
                    let is_selected = self.selected_device.as_ref() == Some(&device);
                    if ui.selectable_label(is_selected, &device).clicked() && !is_selected {
                        self.connect_to_device(device);
                    }
                }
            });

            // メッセージ表示領域
            ui.group(|ui| {
                ui.label("MIDI Messages:");
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for message in &self.messages {
                            ui.label(message);
                        }
                    });
            });
        });

        // 画面の更新を要求
        ctx.request_repaint();
    }
} 