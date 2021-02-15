use std::collections::HashMap;
// use crate::cpu;
#[derive(Copy, Clone)]
pub enum AddressingMode {
    IMM,    // Immediate
    REL,    // Relative
    ZP0,    // ZeroPage
    ZPX,    // ZeroPage X
    ZPY,    // ZeroPage Y
    ABS,    // Absolute
    ABX,    // Absolute X
    ABY,    // Absolute Y
    IND,    // Indirect
    IDX,    // Indirect X
    IDY,    // Indirect Y
    IMP,    // Implied
    ACC,    // Accumulator
}


pub struct Instruction {
    pub name : &'static str,
    pub opcode : u8,
    pub mode: AddressingMode,
    pub bytes: u8,
    pub min_cycles: u8,


}

impl Instruction{
    pub fn new(name : &'static str, opcode : u8 , mode : AddressingMode, bytes :u8, min_cycles : u8) -> Instruction{
        Instruction{
            name,
            opcode,
            mode,
            bytes,
            min_cycles
        }
    }
}

lazy_static!{
    /* see: https://www.masswerk.at/6502/6502_instruction_set.html */

    // '_1_' = +1 cycle if page boundary is crossed

    // '_2_' = +1 to cycles if branch occurs on same page
    //       = +2 to cycles if branch occurs to different page

    pub static ref CPU_INSTRUCTIONS : Vec<Instruction> = vec![
        Instruction::new("BRK", 0x00, AddressingMode::IMP, 1, 7),

        Instruction::new("NOP", 0xEA, AddressingMode::IMP, 1, 2),
        
        // Unofficial Opcodes(to be implemented) 
        Instruction::new("IDK", 0x1C, AddressingMode::IMP, 1, 4),
        Instruction::new("IDK", 0x3C, AddressingMode::IMP, 1, 4),
        Instruction::new("IDK", 0x5C, AddressingMode::IMP, 1, 4),
        Instruction::new("IDK", 0xDC, AddressingMode::IMP, 1, 4),
        Instruction::new("IDK", 0xFC, AddressingMode::IMP, 1, 4),

        // Operations
        Instruction::new("ADC", 0x69, AddressingMode::IMM, 2, 2),
        Instruction::new("ADC", 0x65, AddressingMode::ZP0, 2, 3),
        Instruction::new("ADC", 0x75, AddressingMode::ZPX, 2, 4),
        Instruction::new("ADC", 0x6D, AddressingMode::ABS, 3, 4),
        Instruction::new("ADC", 0x7D, AddressingMode::ABX, 3, 4/* _1_ */),
        Instruction::new("ADC", 0x79, AddressingMode::ABY, 3, 4/* _1_ */),
        Instruction::new("ADC", 0x61, AddressingMode::IDX, 2, 6),
        Instruction::new("ADC", 0x71, AddressingMode::IDY, 2, 5/* _1_ */),

        Instruction::new("AND", 0x29, AddressingMode::IMM, 2, 2),
        Instruction::new("AND", 0x25, AddressingMode::ZP0, 2, 3),
        Instruction::new("AND", 0x35, AddressingMode::ZPX, 2, 4),
        Instruction::new("AND", 0x2D, AddressingMode::ABS, 3, 4),
        Instruction::new("AND", 0x3D, AddressingMode::ABX, 3, 4/* _1_ */),
        Instruction::new("AND", 0x39, AddressingMode::ABY, 3, 4/* _1_ */),
        Instruction::new("AND", 0x21, AddressingMode::IDX, 2, 6),
        Instruction::new("AND", 0x31, AddressingMode::IDY, 2, 5/* _1_ */),

        Instruction::new("ASL", 0x0A, AddressingMode::ACC, 1, 2),
        Instruction::new("ASL", 0x06, AddressingMode::ZP0, 2, 5),
        Instruction::new("ASL", 0x16, AddressingMode::ZPX, 2, 6),
        Instruction::new("ASL", 0x0E, AddressingMode::ABS, 3, 6),
        Instruction::new("ASL", 0x1E, AddressingMode::ABX, 3, 7),

        // Branching
        Instruction::new("BCC", 0x90, AddressingMode::REL, 2, 2 /* _2_ */),
        Instruction::new("BCS", 0xB0, AddressingMode::REL, 2, 2 /* _2_ */),
        Instruction::new("BEQ", 0xF0, AddressingMode::REL, 2, 2 /* _2_ */),
        Instruction::new("BMI", 0x30, AddressingMode::REL, 2, 2 /* _2_ */),
        Instruction::new("BNE", 0xD0, AddressingMode::REL, 2, 2 /* _2_ */),
        Instruction::new("BPL", 0x10, AddressingMode::REL, 2, 2 /* _2_ */),
        Instruction::new("BVC", 0x50, AddressingMode::REL, 2, 2 /* _2_ */),
        Instruction::new("BVS", 0x70, AddressingMode::REL, 2, 2 /* _2_ */),

        Instruction::new("BIT", 0x24, AddressingMode::ZP0, 2, 3),
        Instruction::new("BIT", 0x2C, AddressingMode::ABS, 3, 4),
        
        // Flag Operations
        Instruction::new("CLC", 0x18, AddressingMode::IMP, 1, 2),
        Instruction::new("CLD", 0xD8, AddressingMode::IMP, 1, 2),
        Instruction::new("CLI", 0x58, AddressingMode::IMP, 1, 2),
        Instruction::new("CLV", 0xB8, AddressingMode::IMP, 1, 2),
        
        //
        Instruction::new("CMP", 0xC9, AddressingMode::IMM, 2, 2),
        Instruction::new("CMP", 0xC5, AddressingMode::ZP0, 2, 3),
        Instruction::new("CMP", 0xD5, AddressingMode::ZPX, 2, 4),
        Instruction::new("CMP", 0xCD, AddressingMode::ABS, 3, 4),
        Instruction::new("CMP", 0xDD, AddressingMode::ABX, 3, 4/* _1_ */),
        Instruction::new("CMP", 0xD9, AddressingMode::ABY, 3, 4/* _1_ */),
        Instruction::new("CMP", 0xC1, AddressingMode::IDX, 2, 6),
        Instruction::new("CMP", 0xD1, AddressingMode::IDY, 2, 5/* _1_ */),

        Instruction::new("CPX", 0xE0, AddressingMode::IMM, 2, 2),
        Instruction::new("CPX", 0xE4, AddressingMode::ZP0, 2, 3),
        Instruction::new("CPX", 0xEC, AddressingMode::ABS, 3, 4),

        Instruction::new("CPY", 0xC0, AddressingMode::IMM, 2, 2),
        Instruction::new("CPY", 0xC4, AddressingMode::ZP0, 2, 3),
        Instruction::new("CPY", 0xCC, AddressingMode::ABS, 3, 4),

        Instruction::new("DEC", 0xC6, AddressingMode::ZP0, 2, 5),
        Instruction::new("DEC", 0xD6, AddressingMode::ZPX, 2, 6),
        Instruction::new("DEC", 0xCE, AddressingMode::ABS, 3, 6),
        Instruction::new("DEC", 0xDE, AddressingMode::ABX, 3, 7),

        Instruction::new("DEX", 0xCA, AddressingMode::IMP, 1, 2),
        Instruction::new("DEY", 0x88, AddressingMode::IMP, 1, 2),

        Instruction::new("EOR", 0x49, AddressingMode::IMM, 2, 2),
        Instruction::new("EOR", 0x45, AddressingMode::ZP0, 2, 3),
        Instruction::new("EOR", 0x55, AddressingMode::ZPX, 2, 4),
        Instruction::new("EOR", 0x4D, AddressingMode::ABS, 3, 4),
        Instruction::new("EOR", 0x5D, AddressingMode::ABX, 3, 4/* _1_ */),
        Instruction::new("EOR", 0x59, AddressingMode::ABY, 3, 4/* _1_ */),
        Instruction::new("EOR", 0x41, AddressingMode::IDX, 2, 6),
        Instruction::new("EOR", 0x51, AddressingMode::IDY, 2, 5/* _1_ */),

        Instruction::new("INC", 0xE6, AddressingMode::ZP0, 2, 5),
        Instruction::new("INC", 0xF6, AddressingMode::ZPX, 2, 6),
        Instruction::new("INC", 0xEE, AddressingMode::ABS, 3, 6),
        Instruction::new("INC", 0xFE, AddressingMode::ABX, 3, 7),
        
        Instruction::new("IDX", 0xE8, AddressingMode::IMP, 1, 2),
        Instruction::new("IDY", 0xC8, AddressingMode::IMP, 1, 2),

        Instruction::new("JMP", 0x4C, AddressingMode::ABS, 3, 3),
        Instruction::new("JMP", 0x6C, AddressingMode::IND, 3, 5),

        Instruction::new("JSR", 0x20, AddressingMode::ABS, 3, 6),

        // Store and Loads
        Instruction::new("LDA", 0xA9, AddressingMode::IMM, 2, 2),
        Instruction::new("LDA", 0xA5, AddressingMode::ZP0, 2, 3),
        Instruction::new("LDA", 0xB5, AddressingMode::ZPX, 2, 4),
        Instruction::new("LDA", 0xAD, AddressingMode::ABS, 3, 4),
        Instruction::new("LDA", 0xBD, AddressingMode::ABX, 3, 4/* _1_ */), 
        Instruction::new("LDA", 0xB9, AddressingMode::ABY, 3, 4/* _1_ */),
        Instruction::new("LDA", 0xA1, AddressingMode::IDX, 2, 6),
        Instruction::new("LDA", 0xB1, AddressingMode::IDY, 2, 5/* _2_ */),

        Instruction::new("LDX", 0xA2, AddressingMode::IMM, 2, 2),
        Instruction::new("LDX", 0xA6, AddressingMode::ZP0, 2, 3),
        Instruction::new("LDX", 0xB6, AddressingMode::ZPY, 2, 4),
        Instruction::new("LDX", 0xAE, AddressingMode::ABS, 3, 4),
        Instruction::new("LDX", 0xBE, AddressingMode::ABY, 3, 4/* _1_ */),

        Instruction::new("LDY", 0xA0, AddressingMode::IMM, 2, 2),
        Instruction::new("LDY", 0xA4, AddressingMode::ZP0, 2, 3),
        Instruction::new("LDY", 0xB4, AddressingMode::ZPX, 2, 4),
        Instruction::new("LDY", 0xAC, AddressingMode::ABS, 3, 4),
        Instruction::new("LDY", 0xBC, AddressingMode::ABX, 3, 4/* _1_ */),

        Instruction::new("LSR", 0x4A, AddressingMode::ACC, 1, 2),
        Instruction::new("LSR", 0x46, AddressingMode::ZP0, 2, 5),
        Instruction::new("LSR", 0x56, AddressingMode::ZPX, 2, 6),
        Instruction::new("LSR", 0x4E, AddressingMode::ABS, 3, 6),
        Instruction::new("LSR", 0x5E, AddressingMode::ABX, 3, 7),

        Instruction::new("ORA", 0x09, AddressingMode::IMM, 2, 2),
        Instruction::new("ORA", 0x05, AddressingMode::ZP0, 2, 3),
        Instruction::new("ORA", 0x15, AddressingMode::ZPX, 2, 4),
        Instruction::new("ORA", 0x0D, AddressingMode::ABS, 3, 4),
        Instruction::new("ORA", 0x1D, AddressingMode::ABX, 3, 4/* _1_ */), 
        Instruction::new("ORA", 0x19, AddressingMode::ABY, 3, 4/* _1_ */),
        Instruction::new("ORA", 0x01, AddressingMode::IDX, 2, 6),
        Instruction::new("ORA", 0x11, AddressingMode::IDY, 2, 5/* _2_ */),

        Instruction::new("PHA", 0x48, AddressingMode::IMP, 1, 3),
        Instruction::new("PHP", 0x08, AddressingMode::IMP, 1, 3),
        Instruction::new("PLA", 0x68, AddressingMode::IMP, 1, 4),
        Instruction::new("PLP", 0x28, AddressingMode::IMP, 1, 4),

        Instruction::new("ROL", 0x2A, AddressingMode::ACC, 1, 2),
        Instruction::new("ROL", 0x26, AddressingMode::ZP0, 2, 5),
        Instruction::new("ROL", 0x36, AddressingMode::ZPX, 2, 6),
        Instruction::new("ROL", 0x2E, AddressingMode::ABS, 3, 6),
        Instruction::new("ROL", 0x3E, AddressingMode::ABX, 3, 7),

        Instruction::new("ROR", 0x6A, AddressingMode::ACC, 1, 2),
        Instruction::new("ROR", 0x66, AddressingMode::ZP0, 2, 5),
        Instruction::new("ROR", 0x76, AddressingMode::ZPX, 2, 6),
        Instruction::new("ROR", 0x6E, AddressingMode::ABS, 3, 6),
        Instruction::new("ROR", 0x7E, AddressingMode::ABX, 3, 7),

        Instruction::new("RTI", 0x40, AddressingMode::IMP, 1, 6),
        Instruction::new("RTS", 0x60, AddressingMode::IMP, 1, 6),

        Instruction::new("SBC", 0xE9, AddressingMode::IMM, 2, 2),
        Instruction::new("SBC", 0xE5, AddressingMode::ZP0, 2, 3),
        Instruction::new("SBC", 0xF5, AddressingMode::ZPX, 2, 4),
        Instruction::new("SBC", 0xED, AddressingMode::ABS, 3, 4),
        Instruction::new("SBC", 0xFD, AddressingMode::ABX, 3, 4/* _1_ */), 
        Instruction::new("SBC", 0xF9, AddressingMode::ABY, 3, 4/* _1_ */),
        Instruction::new("SBC", 0xE1, AddressingMode::IDX, 2, 6),
        Instruction::new("SBC", 0xF1, AddressingMode::IDY, 2, 5/* _2_ */),

        Instruction::new("SEC", 0x38, AddressingMode::IMP, 1, 2),
        Instruction::new("SED", 0xF8, AddressingMode::IMP, 1, 2),
        Instruction::new("SEI", 0x78, AddressingMode::IMP, 1, 2),

        Instruction::new("STA", 0x85, AddressingMode::ZP0, 2, 3),
        Instruction::new("STA", 0x95, AddressingMode::ZPX, 2, 4),
        Instruction::new("STA", 0x8D, AddressingMode::ABS, 3, 4),
        Instruction::new("STA", 0x9D, AddressingMode::ABX, 3, 5), 
        Instruction::new("STA", 0x99, AddressingMode::ABY, 3, 5),
        Instruction::new("STA", 0x81, AddressingMode::IDX, 2, 6),
        Instruction::new("STA", 0x91, AddressingMode::IDY, 2, 6),


        Instruction::new("STX", 0x86, AddressingMode::ZP0, 2, 3),
        Instruction::new("STX", 0x96, AddressingMode::ZPY, 2, 4),
        Instruction::new("STX", 0x8E, AddressingMode::ABS, 3, 4),

        Instruction::new("STY", 0x84, AddressingMode::ZP0, 2, 3),
        Instruction::new("STY", 0x94, AddressingMode::ZPX, 2, 4),
        Instruction::new("STY", 0x8C, AddressingMode::ABS, 3, 4),

        Instruction::new("TAX", 0xAA, AddressingMode::IMP, 1, 2),

        Instruction::new("TAY", 0xA8, AddressingMode::IMP, 1, 2),

        Instruction::new("TSX", 0xBA, AddressingMode::IMP, 1, 2),

        Instruction::new("TXA", 0x8A, AddressingMode::IMP, 1, 2),

        Instruction::new("TXS", 0x9A, AddressingMode::IMP, 1, 2),

        Instruction::new("TYA", 0x98, AddressingMode::IMP, 1, 2),



    ];

    pub static ref OPCODE_MAP : HashMap<u8, &'static Instruction> = {
        let vec_ref = &*CPU_INSTRUCTIONS;

        let hmap = vec_ref.iter().map(|x| {(x.opcode, x)}).collect();

        hmap
    };

    pub static ref NAME_MAP : HashMap<&'static str, &'static Instruction> = {
        CPU_INSTRUCTIONS.iter().map(|x| {(x.name, x)}).collect()
    };
    

}
