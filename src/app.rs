use eframe::{egui, epi};

use crate::emulator::cpu::{self, CPU6502};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
// #[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct EmuDisplayApp {
    // Example stuff:
    label: String,
    value: f32,
    painting: Painting,
    pub cpu: CPU6502
}

impl Default for EmuDisplayApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            painting: Default::default(),
            cpu: CPU6502::create_cpu_and_bus()
        }
    }
}

impl epi::App for EmuDisplayApp {
    fn name(&self) -> &str {
        "Simple 6502 Emulator"
    }

    /// Called by the framework to load old app state (if any).
    // #[cfg(feature = "persistence")]
    // fn load(&mut self, storage: &dyn epi::Storage) {
    //     *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
    // }

    /// Called by the frame work to save state before shutdown.
    // #[cfg(feature = "persistence")]
    // fn save(&mut self, storage: &mut dyn epi::Storage) {
    //     epi::set_value(storage, epi::APP_KEY, self);
    // }


    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let EmuDisplayApp {
            label,
            value,
            painting,
            cpu
        } = self;

        cpu.reg_a += 1;
        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        
        egui::SidePanel::left("side_panel", 200.0).show(ctx, |ui| {
            ui.heading("CPU Internal Registers");
            ui.separator();
            ui.label("");
            
            let pc = format!("PC:  ${:04x}", cpu.pc);
            let a = format!("A:    ${:02x}", cpu.reg_a);
            let x = format!("X:    ${:02x}", cpu.reg_x);
            let y = format!("Y:    ${:02x}", cpu.reg_y);
            let sp = format!("SP:  ${:04x}", cpu.stk_ptr as u16 + 0x0100);

            let cpu_internal_label_color = egui::Color32::LIGHT_GRAY;
            
            let register_label = 
                |param : String| {
                    egui::Label::new(param).heading().text_color(cpu_internal_label_color)
                };
            
            let flag_label = 
                |letter : &str, exists: bool| {
                    let color = match exists {
                        true => egui::Color32::GREEN,
                        false => egui::Color32::RED,
                    };
                    egui::Label::new(letter).text_color(color)
                };

            // Status horizontal
            ui.horizontal(|ui| {
                ui.add(egui::Label::new("Status:").text_color(cpu_internal_label_color).heading());
                ui.add(flag_label("N", cpu.status.contains(cpu::Flags::N)));
                ui.add(flag_label("V", cpu.status.contains(cpu::Flags::V)));
                ui.add(flag_label("_", false));
                ui.add(flag_label("B", cpu.status.contains(cpu::Flags::B)));
                ui.add(flag_label("D", cpu.status.contains(cpu::Flags::D)));
                ui.add(flag_label("I", cpu.status.contains(cpu::Flags::I)));
                ui.add(flag_label("Z", cpu.status.contains(cpu::Flags::Z)));
                ui.add(flag_label("C", cpu.status.contains(cpu::Flags::C)));
            });


            ui.add(register_label(pc));
            ui.add(register_label(a));
            ui.add(register_label(x));
            ui.add(register_label(y));
            ui.add(register_label(sp));

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(label);
                
            });

            ui.add(egui::Slider::f32(value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                *value += 1.0;
            }  

            ui.add(egui::SelectableLabel::new(false, "Hello"));
            
            
            

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add(
                    egui::Hyperlink::new("https://github.com/emilk/egui/").text("powered by egui"),
                );
            });
        });

        egui::TopPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("egui template");

            ui.hyperlink("https://github.com/emilk/egui_template");
            ui.add(egui::github_link_file_line!(
                "https://github.com/emilk/egui_template/blob/master/",
                "Direct link to source code."
            ));
            egui::warn_if_debug_build(ui);

            ui.separator();

            ui.add(egui::Label::new("Hamza wrote this").text_color(egui::Color32::RED));

            ui.heading("Central Panel");
            ui.label("The central panel the region left after adding TopPanel's and SidePanel's");
            ui.label("It is often a great place for big things, like drawings:");

            ui.heading("Draw with your mouse to paint:");
            painting.ui_control(ui);
            egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                painting.ui_content(ui);
            });
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("💎Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}

// ----------------------------------------------------------------------------

/// Example code for painting on a canvas with your mouse
// #[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
struct Painting {
    lines: Vec<Vec<egui::Vec2>>,
    stroke: egui::Stroke,
}

impl Default for Painting {
    fn default() -> Self {
        Self {
            lines: Default::default(),
            stroke: egui::Stroke::new(1.0, egui::Color32::LIGHT_BLUE),
        }
    }
}

impl Painting {
    pub fn ui_control(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            egui::stroke_ui(ui, &mut self.stroke, "Stroke");
            ui.separator();
            if ui.button("Clear Painting").clicked() {
                self.lines.clear();
            }
        })
        .response
    }

    pub fn ui_content(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let (response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap_finite(), egui::Sense::drag());
        let rect = response.rect;

        if self.lines.is_empty() {
            self.lines.push(vec![]);
        }

        let current_line = self.lines.last_mut().unwrap();

        if let Some(pointer_pos) = response.interact_pointer_pos() {
            let canvas_pos = pointer_pos - rect.min;
            if current_line.last() != Some(&canvas_pos) {
                current_line.push(canvas_pos);
            }
        } else if !current_line.is_empty() {
            self.lines.push(vec![]);
        }

        let mut shapes = vec![];
        for line in &self.lines {
            if line.len() >= 2 {
                let points: Vec<egui::Pos2> = line.iter().map(|p| rect.min + *p).collect();
                shapes.push(egui::Shape::line(points, self.stroke));
            }
        }
        painter.extend(shapes);

        response
    }
}
