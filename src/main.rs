slint::include_modules!();

use log;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

mod midi;
use midi::{MidiDevice as MidiDeviceImpl, MidiMessage};

#[derive(Debug)]
struct SliderState {
    value: f32,
    is_sending: bool,
}

fn main() -> Result<(), slint::PlatformError> {
    env_logger::init();
    
    let main_window = MainWindow::new()?;
    let midi_device = Arc::new(Mutex::new(MidiDeviceImpl::new()));
    let message_receiver: Arc<Mutex<Option<Receiver<Vec<u8>>>>> = Arc::new(Mutex::new(None));
    let message_sender: Arc<Mutex<Option<Sender<Vec<u8>>>>> = Arc::new(Mutex::new(None));
    let messages = Arc::new(Mutex::new(Vec::with_capacity(1000)));
    let slider_state = Arc::new(Mutex::new(SliderState {
        value: 0.0,
        is_sending: false,
    }));

    // デバイス一覧の更新
    let weak = main_window.as_weak();
    let midi_device_clone = midi_device.clone();
    main_window.on_refresh_devices(move || {
        if let Ok(devices) = MidiDeviceImpl::list_ports() {
            let devices: Vec<_> = devices.into_iter()
                .map(|name| {
                    slint::SharedString::from(name)
                })
                .collect();
            
            let model = slint::VecModel::from(devices);
            weak.unwrap().set_midi_devices(std::rc::Rc::new(model).into());
        }
    });

    // デバイスの選択
    let weak = main_window.as_weak();
    let midi_device_clone = midi_device.clone();
    let message_receiver_clone = message_receiver.clone();
    let message_sender_clone = message_sender.clone();
    main_window.on_device_selected(move |device_name| {
        let (sender, receiver) = mpsc::channel();
        if midi_device_clone.lock().unwrap().connect(&device_name.to_string(), sender.clone()).is_ok() {
            *message_sender_clone.lock().unwrap() = Some(sender);
            *message_receiver_clone.lock().unwrap() = Some(receiver);
        }
    });

    // スライダーの値変更
    let weak = main_window.as_weak();
    let midi_device_clone = midi_device.clone();
    let slider_state_clone = slider_state.clone();
    main_window.on_slider_changed(move |value| {
        let mut state = slider_state_clone.lock().unwrap();
        if (value - state.value).abs() > 0.01 {
            state.is_sending = true;
            if let Err(err) = midi_device_clone.lock().unwrap().send_cc(0, 81, value as u8) {
                log::error!("Failed to send MIDI CC: {}", err);
            }
            state.value = value;
            drop(state);

            // 送信フラグをリセット（50ms後）
            let slider_state_clone = slider_state_clone.clone();
            std::thread::spawn(move || {
                thread::sleep(Duration::from_millis(50));
                let mut state = slider_state_clone.lock().unwrap();
                state.is_sending = false;
                drop(state);
            });
        }
    });

    // MIDIメッセージの受信処理
    let weak = main_window.as_weak();
    let message_receiver_clone = message_receiver.clone();
    let messages_clone = messages.clone();
    let slider_state_clone = slider_state.clone();
    thread::spawn(move || {
        let mut message_buffer = Vec::with_capacity(100);
        loop {
            if let Some(receiver) = &*message_receiver_clone.lock().unwrap() {
                while let Ok(data) = receiver.try_recv() {
                    if let Some(message) = MidiMessage::from_bytes(&data) {
                        message_buffer.push(message);
                    }
                }

                if !message_buffer.is_empty() {
                    let mut messages = messages_clone.lock().unwrap();
                    let mut new_slider_value = None;

                    for message in message_buffer.drain(..) {
                        let msg_str = message.to_string();
                        messages.push(msg_str);
                        if messages.len() > 1000 {
                            messages.remove(0);
                        }

                        // CC#81の値を更新（送信中でない場合のみ）
                        if message.is_control_change() && message.get_controller_number() == 81 {
                            let state = slider_state_clone.lock().unwrap();
                            if !state.is_sending {
                                let value = message.get_value() as f32;
                                new_slider_value = Some(value);
                            }
                            drop(state);
                        }
                    }

                    let messages_vec = messages.clone();
                    drop(messages);

                    if let Some(value) = new_slider_value {
                        let mut state = slider_state_clone.lock().unwrap();
                        state.value = value;
                        drop(state);
                        
                        weak.upgrade_in_event_loop(move |handle| {
                            // メッセージリストの更新
                            let messages: Vec<slint::SharedString> = messages_vec
                                .iter()
                                .map(|s| s.into())
                                .collect();
                            handle.set_messages(std::rc::Rc::new(slint::VecModel::from(messages)).into());
                            
                            // スライダー値の更新
                            handle.set_slider_value(value);
                        }).ok();
                    } else {
                        weak.upgrade_in_event_loop(move |handle| {
                            let messages: Vec<slint::SharedString> = messages_vec
                                .iter()
                                .map(|s| s.into())
                                .collect();
                            handle.set_messages(std::rc::Rc::new(slint::VecModel::from(messages)).into());
                        }).ok();
                    }
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
    });

    main_window.run()
}
