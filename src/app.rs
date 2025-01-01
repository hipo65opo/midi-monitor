use egui::{self, Color32, Slider};
use eframe::{self, NativeOptions};
use log;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use crate::midi::{MidiDevice, MidiMessage};

const BACKGROUND_COLOR: Color32 = Color32::from_rgb(40, 44, 52);
const TEXT_COLOR: Color32 = Color32::from_rgb(220, 223, 228);
const ACCENT_COLOR: Color32 = Color32::from_rgb(97, 175, 239);
const BUTTON_COLOR: Color32 = Color32::from_rgb(97, 175, 239);
const BUTTON_HOVER_COLOR: Color32 = Color32::from_rgb(126, 191, 241);
const DROPDOWN_BG_COLOR: Color32 = Color32::from_rgb(50, 54, 62);
const DROPDOWN_HOVER_COLOR: Color32 = Color32::from_rgb(60, 64, 72);
const TEXTBOX_BG_COLOR: Color32 = Color32::from_rgb(60, 64, 72);
const TEXTBOX_ACTIVE_BG_COLOR: Color32 = Color32::from_rgb(70, 74, 82);

struct MidiMonitorApp {
    midi_device: Arc<Mutex<MidiDevice>>,
    midi_devices: Vec<String>,
    selected_device: Option<String>,
    messages: Vec<String>,
    message_receiver: Option<Receiver<Vec<u8>>>,
    message_sender: Option<Sender<Vec<u8>>>,
    slider_value: f32,
    last_sent_value: i32,
    device_to_connect: Option<String>,
    cc_number: String,
    current_cc: u8,
}

impl MidiMonitorApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // スタイルを設定
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals.window_fill = BACKGROUND_COLOR;
        style.visuals.panel_fill = BACKGROUND_COLOR;
        style.visuals.widgets.noninteractive.fg_stroke.color = TEXT_COLOR;
        style.visuals.widgets.inactive.fg_stroke.color = TEXT_COLOR;
        style.visuals.widgets.active.fg_stroke.color = TEXT_COLOR;
        style.visuals.widgets.hovered.fg_stroke.color = TEXT_COLOR;
        
        // ボタンのスタイル
        style.visuals.widgets.inactive.bg_fill = BUTTON_COLOR;
        style.visuals.widgets.hovered.bg_fill = BUTTON_HOVER_COLOR;
        style.visuals.widgets.active.bg_fill = BUTTON_COLOR;
        style.visuals.widgets.inactive.bg_stroke.color = BUTTON_COLOR;
        style.visuals.widgets.hovered.bg_stroke.color = BUTTON_HOVER_COLOR;
        style.visuals.widgets.active.bg_stroke.color = BUTTON_COLOR;
        
        // コンボボックスのスタイル
        style.visuals.widgets.inactive.weak_bg_fill = DROPDOWN_BG_COLOR;
        style.visuals.widgets.hovered.weak_bg_fill = DROPDOWN_HOVER_COLOR;
        style.visuals.widgets.active.weak_bg_fill = DROPDOWN_BG_COLOR;

        // テキストボックスのスタイル
        style.visuals.extreme_bg_color = TEXTBOX_BG_COLOR;
        style.visuals.code_bg_color = TEXTBOX_BG_COLOR;
        style.visuals.widgets.inactive.bg_fill = TEXTBOX_BG_COLOR;
        style.visuals.widgets.active.bg_fill = TEXTBOX_ACTIVE_BG_COLOR;
        
        cc.egui_ctx.set_style(style);

        Self {
            midi_device: Arc::new(Mutex::new(MidiDevice::new())),
            midi_devices: Vec::new(),
            selected_device: None,
            messages: Vec::new(),
            message_receiver: None,
            message_sender: None,
            slider_value: 0.0,
            last_sent_value: -1,
            device_to_connect: None,
            cc_number: "81".to_string(),
            current_cc: 81,
        }
    }

    fn refresh_devices(&mut self) {
        match MidiDevice::list_ports() {
            Ok(devices) => {
                log::info!("Found {} MIDI devices", devices.len());
                self.midi_devices = devices;
            },
            Err(err) => {
                log::error!("Failed to list MIDI devices: {}", err);
                self.midi_devices.clear();
            }
        }
    }

    fn connect_device(&mut self, device_name: &str) {
        let (sender, receiver) = mpsc::channel();
        match self.midi_device.lock() {
            Ok(mut device) => {
                match device.connect(device_name, sender.clone()) {
                    Ok(_) => {
                        log::info!("Connected to MIDI device: {}", device_name);
                        self.selected_device = Some(device_name.to_string());
                        self.message_sender = Some(sender);
                        self.message_receiver = Some(receiver);
                    },
                    Err(err) => {
                        log::error!("Failed to connect to MIDI device {}: {}", device_name, err);
                        self.selected_device = None;
                    }
                }
            },
            Err(err) => {
                log::error!("Failed to lock MIDI device: {}", err);
                self.selected_device = None;
            }
        }
    }
}

impl eframe::App for MidiMonitorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 保留中のデバイス接続を処理
        if let Some(device_name) = self.device_to_connect.take() {
            self.connect_device(&device_name);
        }

        // MIDIメッセージの受信をチェック
        if let Some(receiver) = &self.message_receiver {
            while let Ok(data) = receiver.try_recv() {
                if let Some(message) = MidiMessage::from_bytes(&data) {
                    // 指定したCCナンバーを受信した場合、スライダーを更新
                    if message.is_control_change() && message.get_controller_number() == self.current_cc {
                        self.slider_value = message.get_value() as f32;
                        self.last_sent_value = message.get_value() as i32;
                        ctx.request_repaint();
                    }
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // 上部パネル
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                // 上部のコントロールパネル
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.heading("MIDI Monitor");
                        ui.add_space(20.0);

                        ui.label("MIDI Devices");
                        let refresh_button = egui::Button::new("🔄 Refresh")
                            .min_size(egui::vec2(120.0, 30.0));
                        if ui.add(refresh_button).clicked() {
                            self.refresh_devices();
                        }
                        ui.add_space(10.0);

                        egui::ComboBox::from_label("")
                            .selected_text(self.selected_device.as_deref().unwrap_or("Select Device"))
                            .width(200.0)
                            .show_ui(ui, |ui| {
                                for device in &self.midi_devices {
                                    if ui.selectable_label(
                                        self.selected_device.as_deref() == Some(device),
                                        device,
                                    ).clicked() {
                                        self.device_to_connect = Some(device.clone());
                                    }
                                }
                            });
                    });
                });
            });

            // 下部パネル
            egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(10.0);
                    
                    // CCナンバー入力
                    ui.horizontal(|ui| {
                        ui.label("CC#");
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.cc_number)
                                .desired_width(50.0)
                                .hint_text("0-127")
                                .text_color(TEXT_COLOR)
                                .frame(true)
                        );
                        
                        if response.lost_focus() {
                            if let Ok(num) = self.cc_number.parse::<u8>() {
                                if num <= 127 {
                                    self.current_cc = num;
                                }
                            }
                        }
                    });
                    
                    // スライダーの値を表示
                    let value_text = format!("{}", self.slider_value as i32);
                    ui.heading(value_text);

                    // スライダー用のコンテナ
                    egui::Frame::none()
                        .fill(BACKGROUND_COLOR)
                        .show(ui, |ui| {
                            let available_width = ui.available_width();
                            
                            // スライダーを含むレイアウト
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center).with_main_justify(true), |ui| {
                                ui.spacing_mut().slider_width = available_width - 20.0; // マージンを考慮
                                ui.spacing_mut().interact_size.y = 4.0;    // つまみの高さ
                                let slider = Slider::new(&mut self.slider_value, 0.0..=127.0)
                                    .orientation(egui::SliderOrientation::Horizontal)
                                    .smart_aim(false)
                                    .show_value(false);

                                ui.add(slider);
                            });
                        });

                    if self.slider_value as i32 != self.last_sent_value {
                        if self.selected_device.is_some() {
                            if let Err(err) = self.midi_device.lock().unwrap()
                                .send_cc(0, self.current_cc, self.slider_value as u8) {
                                log::error!("Failed to send MIDI CC: {}", err);
                            }
                        }
                        self.last_sent_value = self.slider_value as i32;
                    }

                    // CC#ラベル
                    ui.label(format!("CC#{}", self.current_cc));
                    ui.add_space(10.0);
                });
            });
        });

        // 継続的な再描画を要求
        ctx.request_repaint();
    }
}

pub fn run() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "MIDI Monitor",
        options,
        Box::new(|cc| Box::new(MidiMonitorApp::new(cc)))
    )
} 