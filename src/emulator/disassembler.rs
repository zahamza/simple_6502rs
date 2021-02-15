use crate::hex;

use crate::emulator::instruction::{OPCODE_MAP, AddressingMode::*};


pub fn disassemble(obj_code : &Vec<u8>) -> Vec<String>{
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


pub fn disassemble_with_addr_line(obj_code : &Vec<u8>, first_addr : u16) -> Vec<String>
{
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

/// works with proper input only
pub fn string_to_hex(input : &str) -> Vec<u8>{
    
    let processed_input : String = input.split_whitespace().collect();

    hex::decode(processed_input).expect("Decoding failed")
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disassemble(){
        let hex_dump = vec![0xa9, 0x01, 0x8d, 0x00, 0x02, 0xa9, 0x05, 0x8d,
        0x01, 0x02, 0xa9, 0x08, 0x8e, 0x02, 0x02];
        
        let v = disassemble(&hex_dump);

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
        
        let hex_dump = string_to_hex(test);

        let v = disassemble(&hex_dump);

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