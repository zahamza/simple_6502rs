use crate::hex;
use crate::emulator::instruction::{OPCODE_MAP, AddressingMode::*};
use crate::emulator::cpu::CPU6502;

/// designed for GUI use
pub fn disassemble_next_instr(cpu : &CPU6502) -> String {

    let pc = cpu.pc;
    let opcode = cpu.read(pc);

    let (name, mode, bytes) = match OPCODE_MAP.get(&opcode) {
        Some(valid) => (valid.name, valid.mode, valid.bytes),
        None => ("???: Gonna treat as NOP", IMP, 1)
    };

    let mut s = name.to_string();

    let memory = cpu.index_memory(pc, pc+2).unwrap();

    if bytes == 1{
        match mode{
            ACC => {
                s.push_str(" A");
            }
            IMP => (),
            _ => panic!("This opcode ({:x}) shouldn't use 1 byte", opcode)
        };

    } else if bytes == 2 {
        let byte = memory[1];
        match mode {
            IMM => {
                let add = format!(" #${:02x}", byte);
                s.push_str(&add[..]);
            }
            REL | ZP0 => {
                let add = format!(" ${:02x}", byte);
                s.push_str(&add[..]);
            }
            ZPX => {
                let add = format!(" ${:02x},X", byte);
                s.push_str(&add[..]);
            }
            ZPY => {
                let add = format!(" ${:02x},Y", byte);
                s.push_str(&add[..]);
            }
            IDX => {
                let add = format!(" (${:02x},X)", byte);
                s.push_str(&add[..]);
            }
            IDY => {
                let add = format!(" (${:02x}),Y", byte);
                s.push_str(&add[..]);
            }

            _ => panic!("This opcode ({:x}) shouldn't use 2 bytes", opcode)
        };

    } else if bytes == 3 {
        let hi = memory[2];
        let lo = memory[1];

        match mode {
            IND => {
                let add = format!(" (${:02x}{:02x})", hi, lo);
                s.push_str(&add[..]);
            }

            ABS => {
                let add = format!(" ${:02x}{:02x}", hi, lo);
                s.push_str(&add[..]);
            }

            ABX => {
                let add = format!(" ${:02x}{:02x},X", hi, lo);
                s.push_str(&add[..]);
            }

            ABY => {
                let add = format!(" ${:02x}{:02x},Y", hi, lo);
                s.push_str(&add[..]);
            }

            _=> panic!("This opcode ({:x}) shouldn't use 3 bytes", opcode)
        }
    }
    s
}


pub fn disassemble_to_stream(obj_code : &Vec<u8>) -> Vec<String>{
    let mut ret : Vec<String> = Vec::new();

    let mut i = 0;

    while i < obj_code.len() {
        let opcode = obj_code[i];

        let instr = OPCODE_MAP.get(&opcode).unwrap_or_else
            (|| {panic!("At {}th index, found invalid opcode: {}\n", i, opcode)}
            );

        let instr_bytes = instr.bytes;
        let mode = instr.mode;

        let mut s = String::from(instr.name);
        
        if instr_bytes == 1{
            match mode{
                ACC => {
                    s.push_str(" A");
                }
                IMP => (),
                _ => panic!("This opcode ({:x}) shouldn't use 1 byte", opcode)
            };

        } else if instr_bytes == 2 {
            let byte = obj_code[i+1];
            // consumed for byte
            i+=1;

            match mode {
                IMM => {
                    let add = format!(" #${:02x}", byte);
                    s.push_str(&add[..]);
                }
                REL | ZP0 => {
                    let add = format!(" ${:02x}", byte);
                    s.push_str(&add[..]);
                }
                ZPX => {
                    let add = format!(" ${:02x},X", byte);
                    s.push_str(&add[..]);
                }
                ZPY => {
                    let add = format!(" ${:02x},Y", byte);
                    s.push_str(&add[..]);
                }
                IDX => {
                    let add = format!(" (${:02x},X)", byte);
                    s.push_str(&add[..]);
                }
                IDY => {
                    let add = format!(" (${:02x}),Y", byte);
                    s.push_str(&add[..]);
                }

                _ => panic!("This opcode ({:x}) shouldn't use 2 bytes", opcode)
            };


        } else if instr_bytes == 3 {
            let hi = obj_code[i+2];
            let lo = obj_code[i+1];

            // used two bytes for instr;
            i+=2;

            match mode {
                IND => {
                    let add = format!(" (${:02x}{:02x})", hi, lo);
                    s.push_str(&add[..]);
                }

                ABS => {
                    let add = format!(" ${:02x}{:02x}", hi, lo);
                    s.push_str(&add[..]);
                }


                ABX => {
                    let add = format!(" ${:02x}{:02x},X", hi, lo);
                    s.push_str(&add[..]);
                }


                ABY => {
                    let add = format!(" ${:02x}{:02x},Y", hi, lo);
                    s.push_str(&add[..]);
                }

                _=> panic!("This opcode ({:x}) shouldn't use 3 bytes", opcode)
            }
        }
                
        // always have read at least opcode
        i+=1;

        ret.push(s);
        
    }

    ret
}


pub fn disassemble_with_addr_line(obj_code : &Vec<u8>, first_addr : u16) -> Vec<String> {
    let mut ret : Vec<String> = Vec::new();

    let mut i = 0;

    while i < obj_code.len() {
        let opcode = obj_code[i];

        let instr = OPCODE_MAP.get(&opcode).unwrap_or_else
            (|| {panic!("At {}th index, found invalid opcode: {}\n", i, opcode)}
            );

        let instr_bytes = instr.bytes;
        let mode = instr.mode;

        
        let mut s = format!("${:04x}: ", first_addr + i as u16);
        s.push_str(instr.name);

        if instr_bytes == 1{
            match mode{
                ACC => {
                    s.push_str(" A");
                }
                IMP => (),
                _ => panic!("This opcode ({:x}) shouldn't use 1 byte", opcode)
            };

        } else if instr_bytes == 2 {
            let byte = obj_code[i+1];
            // consumed for byte
            i+=1;

            match mode {
                IMM => {
                    let add = format!(" #${:02x}", byte);
                    s.push_str(&add[..]);
                }
                REL | ZP0 => {
                    let add = format!(" ${:02x}", byte);
                    s.push_str(&add[..]);
                }
                ZPX => {
                    let add = format!(" ${:02x},X", byte);
                    s.push_str(&add[..]);
                }
                ZPY => {
                    let add = format!(" ${:02x},Y", byte);
                    s.push_str(&add[..]);
                }
                IDX => {
                    let add = format!(" (${:02x},X)", byte);
                    s.push_str(&add[..]);
                }
                IDY => {
                    let add = format!(" (${:02x}),Y", byte);
                    s.push_str(&add[..]);
                }

                _ => panic!("This opcode ({:x}) shouldn't use 2 bytes", opcode)
            };


        } else if instr_bytes == 3 {
            let hi = obj_code[i+2];
            let lo = obj_code[i+1];

            // used two bytes for instr;
            i+=2;

            match mode {
                IND => {
                    let add = format!(" (${:02x}{:02x})", hi, lo);
                    s.push_str(&add[..]);
                }

                ABS => {
                    let add = format!(" ${:02x}{:02x}", hi, lo);
                    s.push_str(&add[..]);
                }


                ABX => {
                    let add = format!(" ${:02x}{:02x},X", hi, lo);
                    s.push_str(&add[..]);
                }


                ABY => {
                    let add = format!(" ${:02x}{:02x},Y", hi, lo);
                    s.push_str(&add[..]);
                }

                _=> panic!("This opcode ({:x}) shouldn't use 3 bytes", opcode)
            }
        }
                
        // always have read at least opcode
        i+=1;

        ret.push(s);
        
    }

    ret
}

/// hex string to bytes. returns none if there was an error
/// 
pub fn htb_option(input: &str) -> Option<Vec<u8>> {
    let input : String = input.split_whitespace().collect();
    match hex::decode(input){
        Ok(vector) => {
            Some(vector)
        }    
        Err(_) => None
    }
}

/// works with proper input only
pub fn hex_string_to_bytes(input : &str) -> Vec<u8>{
    
    let processed_input : String = input.split_whitespace().collect();

    hex::decode(processed_input).expect("Decoding failed")
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_convert() {

        assert_eq!(0x80 as u8, htb_option("8000").unwrap()[0]);
    }

    #[test]
    fn test_disassemble_stream(){
        let hex_dump = vec![0xa9, 0x01, 0x8d, 0x00, 0x02, 0xa9, 0x05, 0x8d,
        0x01, 0x02, 0xa9, 0x08, 0x8e, 0x02, 0x02];
        
        let v = disassemble_to_stream(&hex_dump);

        let mut result = String::from("");


        // push all except last
        for s in &v[0..v.len()-1] {
            result.push_str(&s[..]);
            result.push_str(" ");            
            print!("{} ",s);

        }
        // push last
        result.push_str(v.last().unwrap());

        let answer = "LDA #$01 STA $0200 LDA #$05 STA $0201 LDA #$08 STX $0202";
        assert_eq!(answer, result);

    }

    #[test]
    fn test_disassemble_with_parse(){

        let test = "a9 01 8d 00 02 a9 05 8d
        01 02 a9 08 8e 02 02";
        
        let hex_dump = hex_string_to_bytes(test);

        let v = disassemble_to_stream(&hex_dump);

        let mut result = String::from("");
        // push all except last
        for s in &v[0..v.len()-1] {
            result.push_str(&s[..]);
            result.push_str(" ");            
            print!("{} ",s);

        }   
        // push last
        result.push_str(v.last().unwrap());

        let answer = "LDA #$01 STA $0200 LDA #$05 STA $0201 LDA #$08 STX $0202";
        assert_eq!(answer, result);
    }
    
}
