use simple_6502rs::app;

fn main() {
    let app = app::EmuDisplayApp::default();
    eframe::run_native(Box::new(app));
}
