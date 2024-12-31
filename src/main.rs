use eframe::NativeOptions;
use env_logger;
use log;

mod app;
mod midi;

fn main() -> Result<(), eframe::Error> {
    // ログ設定の初期化
    env_logger::init();
    log::info!("Starting MIDI Monitor application...");

    let options = NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(800.0, 600.0)),
        ..Default::default()
    };

    // アプリケーションの起動
    eframe::run_native(
        "MIDI Monitor",
        options,
        Box::new(|cc| Box::new(app::App::new(cc)))
    )
}
