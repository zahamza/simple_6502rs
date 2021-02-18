// So its to transfer from workspace
use crate::emulator as emulator;
//
use eframe::{egui, epi};


use emulator::cpu::{self, CPU6502};
use emulator::disassembler::{htb_option};

pub struct EmuDisplayApp {
    pc_change_str : String,
    load_start_str: String,
    read_addr: u16,
    read_addr_input: String,
    write_val: u8,
    write_input: String,
    from_input_1 : String,
    to_input_1: String, 
    from_1 : u16,
    to_1 : u16,
    from_input_2 : String,
    to_input_2: String, 
    from_2 : u16,
    to_2 : u16,
    pub obj_string: String,
    pub cpu: CPU6502,
}

impl Default for EmuDisplayApp {
    fn default() -> Self {
        Self {
            pc_change_str: "8000".to_owned(),
            load_start_str: "8000".to_owned(),
            read_addr: 0x8000,
            read_addr_input: "8000".to_owned(),
            write_val : 0x00,
            write_input: "00".to_owned(),
            from_input_1: "8000".to_owned(),
            to_input_1: "8080".to_owned(),
            from_1: 0x8000,
            to_1: 0x8080,
            from_input_2: "0100".to_owned(),
            to_input_2: "01ff".to_owned(),
            from_2: 0x0100,
            to_2: 0x01ff,
            obj_string: "9A039A2E".to_owned(),
            cpu: CPU6502::create_cpu_and_bus(0x8000),
        }
    }
}


impl epi::App for EmuDisplayApp {
    fn name(&self) -> &str {
        "Simple 6502 Rust Emulation"
    }

    /// One time setup code
    fn setup(&mut self, _ctx: &egui::CtxRef) {

        let mut fonts = egui::FontDefinitions::default();
            fonts.family_and_size.insert(
                egui::TextStyle::Body,
                (egui::FontFamily::Proportional, 16.));
            fonts.family_and_size.insert(
                egui::TextStyle::Heading,
                (egui::FontFamily::Proportional, 22.));
            
        _ctx.set_fonts(fonts);
    }

    /// Main update, should probably refactor
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let EmuDisplayApp {
            pc_change_str,
            load_start_str,
            read_addr,
            read_addr_input,
            write_val,
            write_input,
            from_input_1,
            to_input_1,
            from_1,
            to_1,
            from_input_2,
            to_input_2,
            from_2,
            to_2,
            obj_string,
            cpu,
        } = self;

        // Control Panel
        egui::SidePanel::left("side_panel", 230.0).show(ctx, |ui| {
            ui.heading("Control Panel");
            
            let cpu_internal_label_color = egui::Color32::LIGHT_GRAY; 
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label("Start Address: ");
                
                if ui.text_edit_singleline(load_start_str).lost_kb_focus() {
                    ensure_input(load_start_str, 4);
                }

            });

            ui.separator();
            let scrolling_area = egui::ScrollArea::from_max_height(70.0);

            scrolling_area.show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.text_edit_multiline(obj_string);
                });

            });

            ui.separator();
            
            // Load Program
            ui.vertical_centered(|ui| {
                if ui.button("Load object code").clicked() {

                    match (htb_option(obj_string), htb_option(load_start_str)){
                        // updates memory panel only if valid
                        (Some(obj_code),Some(load_vec)) => {
                            
                            let load_addr = extract_from_hex(load_vec);
                            match cpu.specific_load(obj_code, load_addr) {
                                Ok(_) => (),
                                Err(msg) => *obj_string = msg.into()
                            }

                        }
                        _ => {
                            *obj_string = "Invalid Input".into();
                            *load_start_str = "8000".into();
                        }
                    };
                }
            });

            ui.add(egui::Separator::new().spacing(6.));

            ui.small("");

            // Internal Registers
            ui.vertical_centered(|ui|{
                let pc = format!("PC:  ${:04X}", cpu.pc);
                let a = format!("A:    ${:02X}", cpu.reg_a);
                let x = format!("X:    ${:02X}", cpu.reg_x);
                let y = format!("Y:    ${:02X}", cpu.reg_y);
                let sp = format!("SP:  ${:04X}", cpu.stk_ptr as u16 + CPU6502::STACK_OFFSET);
                let register_label = 
                    |param : &str| {
                        egui::Label::new(param).text_color(cpu_internal_label_color).heading()
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
                    ui.add(register_label("   Status: "));
                    ui.add(flag_label("N", cpu.status.contains(cpu::Flags::N)));
                    ui.add(flag_label("V", cpu.status.contains(cpu::Flags::V)));
                    ui.add(flag_label("_", cpu.status.contains(cpu::Flags::U)));
                    ui.add(flag_label("B", cpu.status.contains(cpu::Flags::B)));
                    ui.add(flag_label("D", cpu.status.contains(cpu::Flags::D)));
                    ui.add(flag_label("I", cpu.status.contains(cpu::Flags::I)));
                    ui.add(flag_label("Z", cpu.status.contains(cpu::Flags::Z)));
                    ui.add(flag_label("C", cpu.status.contains(cpu::Flags::C)));
                });
                ui.add(register_label(&pc));
                ui.add(register_label(&a));
                ui.add(register_label(&x));
                ui.add(register_label(&y));
                ui.add(register_label(&sp));
            });

            // Command Buttons
            ui.vertical_centered(|ui|{
                ui.spacing_mut().item_spacing = egui::math::vec2(0., 7.);

                ui.label("");
                if ui.button("Next Step").clicked() {
                    cpu.execute_step();
                }
                if ui.button("Continuous Run").clicked(){
                    cpu.run_until_brk();
                }
                if ui.button("CPU Reset").clicked(){
                    cpu.reset();
                    cpu.run_cycles(CPU6502::RESET_CYCLES);
                }
                if ui.button("Clear CPU").clicked(){
                    *cpu = CPU6502::create_cpu_and_bus(0x8000);
                }

            });

            ui.label("");
            ui.separator();
            ui.horizontal(|ui| {
                let pc_button =  ui.button("Load PC: ");
                
                if ui.text_edit_singleline(pc_change_str).lost_kb_focus() {
                    ensure_input(pc_change_str, 4);
                }

                if pc_button.clicked(){
                    if let Some(pc_vec) = htb_option(pc_change_str) {
                        cpu.pc = extract_from_hex(pc_vec);
                    }
                }

            });

            ui.separator();

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add(
                    egui::Hyperlink::new("https://github.com/emilk/egui/").text("powered by egui"),
                );

                ui.label("");
            });
        });

        // Top "Bar"
        // TO DO: Add sources/credits here
        egui::TopPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });

                ui.separator();

                egui::menu::menu(ui, "More 6502", |ui| {
                    ui.add(
                        egui::Hyperlink::new("https://skilldrick.github.io/easy6502/").text("Assembly Guide")
                    );
                    ui.add(
                        egui::Hyperlink::new("http://obelisk.me.uk/6502/reference.html").text("Instruction Reference")
                    );
                    ui.add(
                        egui::Hyperlink::new("https://github.com/zahamza/simple_6502rs").text("Instruction Set")
                    );

                });

                ui.separator();

                ui.add(egui::Hyperlink::new("https://github.com/zahamza/simple_6502rs").text("Source Code"));


                ui.separator();
            });
    
        });

        // Contains memory panels and some text
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Introduction");
            ui.horizontal_wrapped_for_text(egui::TextStyle::Body, |ui|{
                ui.label("Hi! I made this for for fun while making a NES emulator, so decimal mode is unsupported.");
                ui.label("Remember, numbers (including inputs) are/should be hexadecimal.");
            });

            ui.horizontal_wrapped_for_text(egui::TextStyle::Body, |ui|{
                ui.label("Check");
                ui.add(
                    egui::Hyperlink::new("https://github.com/zahamza/simple_6502rs/blob/main/README.md").text("README.md")
                );
                ui.label("for more details.");
            });

            egui::warn_if_debug_build(ui);

            ui.separator();

            ui.heading("Main Panel");
            ui.horizontal_wrapped_for_text(egui::TextStyle::Body, |ui|{
                ui.add(
                    egui::Hyperlink::new("https://www.masswerk.at/6502/assembler.html").text("Assembler Link."),
                );
                ui.label("Any object code should work.");
            });

            ui.label("");

            const ADDR_LINE_COLOR : egui::Color32 = egui::Color32::WHITE;

            // Address line sectio
            ui.horizontal_wrapped_for_text(egui::TextStyle::Body, |ui| {

                ui.add(egui::Label::new("Address:").text_color(ADDR_LINE_COLOR));
                
                ui.set_width(ADDR_INPUT_WIDTH);
                if ui.text_edit_singleline(read_addr_input).lost_kb_focus() {
                    ensure_input(read_addr_input, 4);
                    
                }

                ui.set_max_width(500.);
                

                ui.separator();
                // read seciton

                if let Some(addr_vec) = htb_option(read_addr_input){
                    *read_addr = extract_from_hex(addr_vec);
                }

                ui.colored_label(ADDR_LINE_COLOR,format!("Read: ${:02x}", cpu.read(*read_addr)));
                
                

                ui.separator();
                
                // write section
                let write_button = ui.add(egui::Button::new("Write").small().text_color(ADDR_LINE_COLOR));
                
                ui.set_width(ADDR_INPUT_WIDTH/2.);
                if ui.text_edit_singleline(write_input).lost_kb_focus() {
                    ensure_input(write_input, 2);
                    
                }

                ui.set_max_width(500.);

                if let Some(addr_vec) = htb_option(read_addr_input){
                    *read_addr = extract_from_hex(addr_vec);
                }

                if write_button.clicked() {

                    *write_val = match htb_option(write_input) {
                        Some(write_vec) => 
                            extract_from_hex(write_vec) as u8,

                        None => *write_val
                    };
                    cpu.write(*read_addr, *write_val)
                }

                
            });

            ui.label("");

            // ==========================
            ui.heading("Next Instruction:");
            
            let disassembled_str = emulator::disassembler::disassemble_next_instr(cpu);
            ui.separator();
            ui.add(egui::Label::new(disassembled_str).heading().text_color(egui::Color32::LIGHT_GRAY).italics());
            ui.separator();
            ui.label("");
            // ==========================

            const MEM_SCROLL_HEIGHT : f32 = 70.;
            const ADDR_INPUT_WIDTH : f32 = 40.;
            // Memory Panel Inputs 1
            ui.horizontal(|ui| {
                ui.heading("Memory Panel 1: ");

                ui.horizontal(|ui|{
                    ui.set_width(ADDR_INPUT_WIDTH);
                    if ui.text_edit_singleline(from_input_1).lost_kb_focus() {
                        ensure_input(from_input_1, 4);
                    }
                });
                
                ui.label(" to ");

                ui.horizontal(|ui|{
                    ui.set_width(ADDR_INPUT_WIDTH);
                    if ui.text_edit_singleline(to_input_1).lost_kb_focus() {
                        ensure_input(to_input_1, 4);
                    }
                });

                // load memory panel
                if ui.button("Load").clicked(){
                    match (htb_option(from_input_1), htb_option(to_input_1)){
                        (Some(from_vec), Some(to_vec)) => {
                            let from_test = extract_from_hex(from_vec);
                            let to_test = extract_from_hex(to_vec);
                            if from_test < to_test{
                                *from_1 = from_test;
                                *to_1 = to_test;
                            }
                        }
                        _ => {
                            *from_input_1 = "0000".into();
                            *to_input_1 = "0000".into();
                        }
                    } 
                }

            });

            ui.separator();
            let scrolling_area = egui::ScrollArea::from_max_height(MEM_SCROLL_HEIGHT).
                id_source("second_memory_area");
            // scrolling panel one
            scrolling_area.always_show_scroll(true).show(ui, |ui| {
                draw_panel_rows_16wide(ui, cpu, *from_1, *to_1);
            });
            ui.separator();

            ui.label("");
            // Memory Panel Inputs 2
            ui.horizontal(|ui| {
                ui.heading("Memory Panel 2: ");

                ui.horizontal(|ui|{
                    ui.set_width(ADDR_INPUT_WIDTH);
                    if ui.text_edit_singleline(from_input_2).lost_kb_focus() {
                        ensure_input(from_input_2, 4);
                    }
                });
                
                ui.label(" to ");

                ui.horizontal(|ui|{
                    ui.set_width(ADDR_INPUT_WIDTH);
                    if ui.text_edit_singleline(to_input_2).lost_kb_focus() {
                        ensure_input(to_input_2, 4);
                    }
                });
                
                // load memory panel
                if ui.button("Load").clicked(){
                    match (htb_option(from_input_2), htb_option(to_input_2)){
                        (Some(from_vec), Some(to_vec)) => {
                            let from_test = extract_from_hex(from_vec);
                            let to_test = extract_from_hex(to_vec);
                            if from_test < to_test{
                                *from_2 = from_test;
                                *to_2 = to_test;
                            }
                        }
                        _ => {
                            *from_input_2 = "0000".into();
                            *to_input_2 = "0000".into();
                        }
                    } 
                }

            });

            ui.separator();
            let scrolling_area = egui::ScrollArea::from_max_height(MEM_SCROLL_HEIGHT);
            scrolling_area.always_show_scroll(true).show(ui, |ui| {
                draw_panel_rows_16wide(ui, cpu, *from_2, *to_2);
            });
            ui.separator();
        });

    }
}


/// takes in hex bytes from addr_input and outputs addr
fn extract_from_hex(hex_vec : Vec<u8>) -> u16{
    let mut load_addr: u16 = 0;
    let len = hex_vec.len();
    for i in 0..len{
        load_addr |= (hex_vec[i] as u16) << ((len-1-i)*8);
    }   

    load_addr
}


/// Pads 0's to front if needed (this will still be considered valid)
///
/// If input is invalid (too long or contains non-hex character) turns input to "0000"
/// returns true if input is valid
fn ensure_input(input : &mut String, desired_len: usize) -> bool{
    let len = input.len();

    let mut ret = true;

    if len > desired_len {
        *input = "0".repeat(desired_len);
        ret = false;
    } else if len < desired_len {
        let mut s = "0".repeat(desired_len - len);
        s.push_str(input);
        *input = s;
    }

    if htb_option(input) == None {
        *input = "0".repeat(desired_len);
        ret = false;
    }

    ret
} 

/// draw panel rows as specified
fn draw_panel_rows_16wide(ui : &mut egui::Ui, cpu: &CPU6502, from: u16, to: u16){
    ui.vertical(|ui| {
        
        let mut i = 0;
        let mut current_row :u16 = from & 0xfff0;

        const SPACING : egui::Vec2 = egui::vec2(0., 4.);
        ui.spacing_mut().item_spacing = SPACING;

        while current_row < to {
            let mem_slice = cpu.index_memory(current_row, current_row+0xf).unwrap();

            let s : String = mem_slice.iter().take(15).map(|i| format!("{:02X}    ", i)).collect();
            let row = format!("{:04X}:   {}{:02X} ", current_row, s, mem_slice.last().unwrap());
     
            if i & 1 == 0 {
                ui.add(egui::Label::new(&row[..])
                    .background_color(egui::Color32::from_black_alpha(250)));
            } else {
                ui.label(&row[..]);
            }

            i+=1;
            current_row += 0x10;
        }
        
    });
}   
