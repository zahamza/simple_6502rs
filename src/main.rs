// use simple_6502rs::emulator::cpu::CPU6502;
use simple_6502rs::app;

fn main() {
    let app = app::EmuDisplayApp::default();
    // app.cpu.reg_x = 20;
    // app.cpu.reg_y = 0xfe;
    eframe::run_native(Box::new(app));
}
