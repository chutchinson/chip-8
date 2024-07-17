#[macro_use] mod log;
mod cpu;
mod gpu;
mod timer;
mod chip;
mod keypad;
mod speaker;
mod gui;
mod asm;

use std::fs::File;
use std::io::Read;
use chip::Chip;

use eframe::egui;
use egui::{Align, Color32, Direction, Frame, Label, Response, RichText, Rounding, Sense, Stroke, TextStyle, Ui, Vec2, Widget};
use egui_extras::{Column, TableBuilder};
use crate::asm::Disassembler;
use crate::gui::viewport::viewport;

struct App {
    chip: Chip,
    cycle: usize,
    autostep: bool
}

impl Default for App {
    fn default() -> Self {
        // let filename = std::env::args().nth(1).unwrap();
        let filename = r#"F:\Projects\chip-8\roms\IBM Logo.ch8"#;
        //let filename = r#"C:\Users\chris\Downloads\4-flags.ch8"#;
        let mut chip = Chip::new();
        let mut rom = Vec::new();
        let mut file = File::open(filename).unwrap();
        let _ = file.read_to_end(&mut rom).unwrap();

        chip.load(&rom);
        Self {
            chip,
            autostep: false,
            cycle: 0
        }
    }
}

fn colored_label(text: String, color: Color32) -> impl Widget {
    move |ui: &mut egui::Ui| {
        let mut frame = Frame::none()
            .fill(color)
            .inner_margin(4.0);
        let response = frame.show(ui, |ui| {
            ui.label(text);
        });
        response.response
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        if self.autostep {
            self.chip.cycle();
        }

        let screen = self.chip.gpu.vram;

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Grid::new("grid")
                .spacing([20.0, 20.0])
                .num_columns(2)
                .show(ui, |ui| {

                    ui.add(viewport(screen, 2.0));

                    // Disassembly
                    let disassembler = Disassembler::default();
                    let rom = &self.chip.cpu.memory[0x200..];

                    egui::Grid::new("disassembly")
                        .striped(true)
                        .min_col_width(ui.max_rect().width())
                        .show(ui, |ui| {
                            let count = 12;
                            let offset = (self.chip.cpu.pc - 0x200) % count;
                            let start = offset;
                            let end = (offset + count).min(self.chip.cpu.memory.len() as u16);
                            println!("{} {}", start, end);
                            for x in start..end {
                                if let Some(instr) = disassembler.disassemble(&rom[x as usize..]) {
                                    if self.chip.cpu.pc - 0x200 == x {
                                        let fill = Color32::from_rgba_unmultiplied(255, 0, 0, 50);
                                        ui.add(colored_label(format!("{}", instr), fill));
                                    } else {
                                        let fill = Color32::TRANSPARENT;
                                        ui.add(colored_label(format!("{}", instr), fill));
                                    }
                                    ui.end_row();
                                }
                            }
                        });
                    // ui.push_id("disassembly", |ui| {
                    //     TableBuilder::new(ui)
                    //         .column(Column::remainder())
                    //         .striped(true)
                    //         .vscroll(true)
                    //         .sense(Sense::hover())
                    //         .body(|mut body| {
                    //             for x in 0..512 {
                    //                 if let Some(instr) = disassembler.disassemble(&rom[x..]) {
                    //                     body.row(20.0, |mut row| {
                    //                         let index = 0x200u16 + x as u16;
                    //                         row.col(|ui| { ui.label(format!("{}", instr)); });
                    //                         row.set_selected(true);
                    //                     });
                    //                 }
                    //             }
                    //         });
                    //});

                    ui.end_row();

                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.button("Load");

                            let run = if !self.autostep { "Run" } else { "Stop" };
                            if ui.button(run).clicked() {
                                self.autostep = !self.autostep;
                            }

                            if ui.button("Step").clicked() {
                                self.chip.cycle();
                            }
                        });
                    });

                    ui.vertical(|ui| {
                        TableBuilder::new(ui)
                            .column(Column::remainder())
                            .column(Column::remainder())
                            .striped(true)
                            .vscroll(false)
                            .body(|mut body| {
                                for (idx, value) in self.chip.cpu.v.iter().enumerate() {
                                    body.row(20.0, |mut row| {
                                        row.col(|ui| { ui.label(format!("v{idx:x}")); });
                                        row.col(|ui| { ui.label(format!("0x{value:02x}")); });
                                    });
                                }
                            });
                    });

                    ui.end_row();
            });
        });
    }
}
fn main() {
    // Chip::execute().unwrap();

    let viewport = egui::ViewportBuilder::default()
        .with_inner_size([800.0, 600.0])
        .with_close_button(true);

    let options = eframe::NativeOptions {
        centered: true,
        viewport, ..Default::default()
    };

    eframe::run_native("chip-8", options,
        Box::new(|ctx| {
            ctx.egui_ctx.style_mut(|style| {
                style.spacing.button_padding = Vec2::new(6.0, 6.0);
            });
            Box::<App>::default()
        }))
        .expect("Failed to launch user interface");
}