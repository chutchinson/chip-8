#[macro_use] mod log;
mod cpu;
mod gpu;
mod timer;
mod chip;
mod keypad;
mod speaker;

use chip::Chip;

// use eframe::egui;

// struct App;

// impl Default for App {
//     fn default() -> Self {
//         Self {}
//     }
// }

// impl eframe::App for App {
//     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//         egui::CentralPanel::default().show(ctx, |ui| {
//             ui.heading("chip-8");
//         });
//     }
// }

fn main() {
    Chip::execute().unwrap();

    // let options = eframe::NativeOptions {
    //     viewport: egui::ViewportBuilder::default()
    //         .with_inner_size([640.0, 480.0]),
    //         ..Default::default()
    // };
    // eframe::run_native("chip-8", options, 
    //     Box::new(|ctx| {
    //         Box::<App>::default()
    //     }))
    //     .expect("Failed to launch user interface");
}