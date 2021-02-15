#[cfg(test)]
mod tests {
    use crate::emulator::cpu::*;
    use crate::emulator::bus;
    use crate::emulator::instruction::NAME_MAP;

    const RESET_CYCLES : u32 = 8;

    fn get_opcode_from_name(name :&'static str) -> u8 {
        let instr = NAME_MAP.get(name);

        match instr {
            Some(x) => x.opcode,

            None => panic!("Name: {} does not correspond to an opcode, remember names must be all caps", name)
        }

    }

    #[test]
    fn flag_testing() {
        let mut status = Flags::NONE;
        status.set(Flags::C, true);
        status.set(Flags::I, true);
        status.set(Flags::C, true);
        status.set(Flags::Z, true);
        assert_eq!(status, Flags::C | Flags::I | Flags::Z);
    }
    #[test]
    fn load_testing(){
        let bus = Box::new(bus::Bus::new());
        let mut cpu = CPU6502::new(bus);

        match cpu.load(vec![5, 0xa9]) {
            Err(x) => panic!("Error: {}", x),

            _ => ()
        };

        assert_eq!(cpu.pc, 0x8000);
        assert_eq!(cpu.read(0x8000), 5);
        assert_eq!(cpu.read(0x8001), 0xa9);

    }

    #[test]
    fn lda_imm() {
        let bus = Box::new(bus::Bus::new());
        let mut cpu = CPU6502::new(bus);

        let y = cpu.load(vec![0xa9, 255]);

        match y {
            Err(x) => panic!("Error: {}", x),

            _ => ()
        };

        cpu.run_cycles(2 + RESET_CYCLES);

        assert_eq!(cpu.reg_a, 255);
        assert_eq!(cpu.status.contains(Flags::N), true);

    }

    #[test]
    fn lda_abs() {
        let bus = Box::new(bus::Bus::new());
        let mut cpu = CPU6502::new(bus);

        let data = 203;

        let data_addr = 0x7778;

        cpu.write(data_addr, data);

        let y = cpu.load(vec![0xad, 0x78, 0x77]);

        match y {
            Err(x) => panic!("Error: {}", x),

            _ => ()
        };

        cpu.run_cycles(4 + RESET_CYCLES);

        assert_eq!(cpu.reg_a, data);
    }

    #[test]
    fn ld_abx() {

        let bus = Box::new(bus::Bus::new());
        let mut cpu = CPU6502::new(bus);

        let x : u8 = 5;
        
        let addr = 0x6000;
        let addr_lo = addr as u8;
        let addr_hi = (addr >> 8) as u8;

        let y = 54;
        cpu.write(addr + (x as u16) , y);

        let pro = vec![0xa2, x, 0xbc, addr_lo, addr_hi];

        let cycles = 2 + 4;

        match cpu.load(pro) {
            Err(x) => panic!("{}",x),
            Ok(()) => ()
        };

        cpu.run_cycles(cycles + RESET_CYCLES);

        assert_eq!(cpu.reg_x, x);
        assert_eq!(cpu.reg_y, y);
    }

    #[test]
    fn ld_aby_page_crossing() {
        let bus = Box::new(bus::Bus::new());
        let mut cpu = CPU6502::new(bus);

        let y : u8 = 5;
        
        let addr = 0x7000 - (y as u16);
        let addr_lo = addr as u8;
        let addr_hi = (addr >> 8) as u8;

        let x = 54;
        cpu.write(addr + (y as u16) , x);

        let bad_y = 200;

        let pro = vec![0xa0, y, 0xbe, addr_lo, addr_hi, 0xa0, bad_y];

        // enough cycles for first two instrs
        let cycles = 2 + (4+1);

        match cpu.load(pro) {
            Err(z) => panic!("{}", z),
            Ok(()) => ()
        };

        cpu.run_cycles(cycles + RESET_CYCLES);

        assert_eq!(cpu.reg_x, x);
        // there shouldnt be enough cycles to ld bad_y
        assert_ne!(cpu.reg_y, bad_y);

    }


    #[test]
    fn branch_test_carry(){
        let bus = Box::new(bus::Bus::new());
        let mut cpu = CPU6502::new(bus);


        let jmp:i8 = -1*0xa;
        let byte_jmp = jmp as u8;
        // SEC called first
        let pro = vec![0x38, 0x90, 0x0, 0x18, 0x90, byte_jmp];

        match cpu.load(pro) {
            Err(z) => panic!("{}", z),
            Ok(()) => ()
        };

        // run initial
        cpu.run_cycles(RESET_CYCLES);
        assert!(!cpu.status.contains(Flags::C));
        // run SEC
        cpu.run_cycles(2);
        assert!(cpu.status.contains(Flags::C));

        // run BCC (wont jump)
        cpu.run_cycles(2);

        // run CLC
        cpu.run_cycles(2);

        assert_eq!(0x8004, cpu.pc);

        // run working BCC
        cpu.run_cycles(2+1);

        let addr_reached = 0x7ffc;//(0x8006 as u16).wrapping_add(jmp as u16);
        assert_eq!(cpu.pc, addr_reached);
    }


    #[test]
    fn branch_test_page_crossing(){
        let bus = Box::new(bus::Bus::new());
        let mut cpu = CPU6502::new(bus);


        let jmp : i8 = -1*0x7;
        let byte_jmp = jmp as u8;
        // SEC called first
        let pro = vec![0x38, 0x90, 0x0, 0x18, 0x90, byte_jmp];

        cpu.write(0x7fff, 0x38);

        match cpu.load(pro) {
            Err(z) => panic!("{}", z),
            Ok(()) => ()
        };


        cpu.run_cycles(RESET_CYCLES);
        assert!(!cpu.status.contains(Flags::C));
        // run SEC
        cpu.run_cycles(2);
        assert!(cpu.status.contains(Flags::C));

        // run BCC (wont jump)
        cpu.run_cycles(2);

        // run CLC
        cpu.run_cycles(2);

        assert_eq!(0x8004, cpu.pc);

        // run instr but not move pc as not done due to page_crossing
        cpu.run_cycles(2+1);

        // should be 0x7fff if page_crossed
        let addr_reached = 0x7fff;//(0x8006 as u16).wrapping_add(jmp as u16);

        assert_eq!(cpu.pc, addr_reached);
        cpu.run_cycles(1);
        // SEC from 0x7fff shouldnt have been called due to page_crossing extra cycle
        assert!(!cpu.status.contains(Flags::C));

        // run 0x7fff SEC
        cpu.run_cycles(2);
        assert!(cpu.status.contains(Flags::C));


    }

    #[test]
    fn adc_test() {
        let bus = Box::new(bus::Bus::new());
        let mut cpu = CPU6502::new(bus);

        let adc_imm = 0x69;
        let val = 252;
        let start:u8 = 240;

        let overflow_val = 80;
        
        let pro = vec![adc_imm, val, adc_imm,overflow_val];

        match cpu.load(pro) {
            Err(z) => panic!("{}", z),
            Ok(()) => ()
        };
        cpu.run_cycles(RESET_CYCLES);

        cpu.reg_a = start;

        cpu.run_cycles(2);
        // check result
        assert_eq!(cpu.reg_a, start.wrapping_add(val));

        // check flags
        assert!(!cpu.status.contains(Flags::V));
        assert!(cpu.status.contains(Flags::C));
        assert!(cpu.status.contains(Flags::N));

        // (remember carry flag has been set from prev instr)
        let v = overflow_val;
        cpu.reg_a = v; // 80
        cpu.run_cycles(2);
        assert_eq!(cpu.reg_a, v.wrapping_add(1).wrapping_add(v));
        assert!(cpu.status.contains(Flags::V));
        assert!(cpu.status.contains(Flags::N));
        assert!(!cpu.status.contains(Flags::C));
    }

    #[test]
    fn sbc_test() {
        // remember, carry flag set
        // means no borrow
        let bus = Box::new(bus::Bus::new());
        let mut cpu = CPU6502::new(bus);

        let sbc_imm = 0xE9;
        let start:u8 = 24;
        let val = 20;
        let val_2 = 28;
        
        let val_3 = 112;

        let pro = vec![sbc_imm, val, sbc_imm, val_2, sbc_imm, val_3];

        match cpu.load(pro) {
            Err(z) => panic!("{}", z),
            Ok(()) => ()
        };
        cpu.run_cycles(RESET_CYCLES);

        cpu.reg_a = start;
        // set to indicate no prev borrow
        cpu.status.set(Flags::C, true);
        cpu.run_cycles(2);
        // check result
        assert_eq!(cpu.reg_a, start.wrapping_sub(val));
        // no borrow
        assert!(cpu.status.contains(Flags::C));
        assert!(!cpu.status.contains(Flags::V));

        // testing borrowing
        cpu.status.set(Flags::C, false);
        cpu.reg_a = start;
        cpu.run_cycles(2);
        assert_eq!(cpu.reg_a, start.wrapping_sub(val_2) - 1);
        // borrow occured during op
        assert!(!cpu.status.contains(Flags::C));
        assert!(!cpu.status.contains(Flags::V));

        // testing overflow
        let start = 208;
        cpu.reg_a = start;
        // set to indicate no prev borrow
        cpu.status.set(Flags::C, true);
        cpu.run_cycles(2);
        assert_eq!(cpu.reg_a, start.wrapping_sub(val_3));
        // borrow didnt occur during op
        assert!(cpu.status.contains(Flags::C));
        assert!(cpu.status.contains(Flags::V));

    }

    #[test]
    fn addrmode_sta_tst() {

        let bus = Box::new(bus::Bus::new());
        let mut cpu = CPU6502::new(bus);

        let sta_zp0 = 0x85;
        let sta_zpx = 0x95;
        let sta_ab = 0x8d;
        let sta_abx = 0x9d;
        let sta_aby = 0x99;
        let sta_inx = 0x81;
        let sta_iny = 0x91;

        let pro = vec![sta_zp0, 0x0, sta_zpx, 0x00, sta_ab, 0x02, 0x00, sta_abx, 0x00, 0x11,
                        sta_aby, 0x00, 0x12, sta_inx, 0x00, sta_iny, 0x00];

        match cpu.load(pro) {
            Err(z) => panic!("{}", z),
            Ok(()) => ()
        };
        cpu.run_cycles(RESET_CYCLES);

        let a = 0x10;
        let x = 1;
        let y = 3;
        cpu.reg_x = x;
        cpu.reg_y = y;
        cpu.reg_a = a;

        // should be enough for all instructions
        cpu.run_cycles(3+4+4+5+5);

        assert_eq!(cpu.read(0), a);
        assert_eq!(cpu.read(0x00+ x as u16), a);
        assert_eq!(cpu.read(0x0002), a);
        assert_eq!(cpu.read(0x1100+ x as u16), a);
        assert_eq!(cpu.read(0x1200+ y as u16), a);

        // indirect tests

        // indirect_x reads addr bytes from 0001-0002
        cpu.run_cycles(6);
        assert_eq!(cpu.read(0x1010), a);

        // indirect_y read addr bytes from 0000-0001
        cpu.run_cycles(6);
        assert_eq!(cpu.read(0x1010+ y as u16), a);


    }

    // Incomplete tests 
    #[test]
    fn jmp_test(){
        let bus = Box::new(bus::Bus::new());
        let mut cpu = CPU6502::new(bus);

        let jmp_abs = 0x4c;
        let jmp_ind = 0x6c;

        let pro = vec![jmp_abs, 0x05, 0x80, 0x0, 0x0, jmp_ind, 0x01, 0x80];

        match cpu.load(pro) {
            Err(z) => panic!("{}", z),
            Ok(()) => ()
        };
        cpu.run_cycles(RESET_CYCLES);

        // jmp_abs tested
        cpu.run_cycles(3);
        assert_eq!(cpu.pc, 0x8005);

        // jmp_ind tested
        cpu.run_cycles(5);
        assert_eq!(cpu.pc, 0x8005);
        assert_eq!(cpu.pc, cpu.read_u16(0x8001));
    }
    
    #[test]
    fn stack_test() {
        let bus = Box::new(bus::Bus::new());
        let mut cpu = CPU6502::new(bus);

        let pha = 0x48;
        let pla = 0x68;
        let php = 0x08;
        let plp = 0x28;
        let txs = 0x9a;

        let pro = vec![txs, pha, php, plp, pla];

        match cpu.load(pro) {
            Err(z) => panic!("{}", z),
            Ok(()) => ()
        };
        cpu.run_cycles(RESET_CYCLES);

        cpu.reg_x = 0xff;
        // run tsx
        cpu.run_cycles(2);
        assert_eq!(cpu.reg_x, cpu.stk_ptr);

        let val = 200;

        cpu.reg_a = val;
        cpu.status = Flags::D;

        // run pushing
        cpu.run_cycles(3+3);
        assert_eq!(cpu.read(0x01ff), val);
        assert_eq!(cpu.read(0x01fe), Flags::D.bits());

        // change registers to test pull
        cpu.reg_a = 0;
        cpu.status.set(Flags::D, false);

        cpu.run_cycles(4+4);
        assert_eq!(cpu.reg_a, val);
        assert!(cpu.status.contains(Flags::D));

        assert_eq!(cpu.stk_ptr, 0xff);

    }

    #[test]
    fn jsr_rts_test(){
        let bus = Box::new(bus::Bus::new());
        let mut cpu = CPU6502::new(bus);

        let jsr = get_opcode_from_name("JSR");
        let rts = 0x60;

        let pro = vec![jsr, 0x05, 0x80, 0x00, 0x00, rts];

        match cpu.load(pro) {
            Err(z) => panic!("{}", z),
            Ok(()) => ()
        };
        cpu.run_cycles(RESET_CYCLES);
        // instead of tsx, manually
        cpu.stk_ptr = 0xff;

        // run jsr
        cpu.run_cycles(6);
        assert_eq!(cpu.pc, 0x8005);
        assert_eq!(cpu.read_u16(0x01fe), 0x8002);
        assert_eq!(cpu.stk_ptr, 0xfd);

        // run rts
        cpu.run_cycles(6);
        assert_eq!(cpu.pc, 0x8003);
        assert_eq!(cpu.stk_ptr, 0xff);
    }


    #[test]
    fn brk_test(){
        let bus = Box::new(bus::Bus::new());
        let mut cpu = CPU6502::new(bus);

        let nop = get_opcode_from_name("NOP");
        let brk = get_opcode_from_name("BRK");
        let rti = get_opcode_from_name("RTI");

        let pro = vec![nop, brk, 0, 0, rti];

        match cpu.load(pro) {
            Err(z) => panic!("{}", z),
            Ok(()) => ()
        };
        cpu.run_cycles(RESET_CYCLES);
        cpu.stk_ptr = 0xff;

        // nop ran
        cpu.run_cycles(2);
        assert_eq!(cpu.pc, 0x8001);

        // change starting value for brk
        cpu.write(0xFFFE, 0x04);
        cpu.write(0xFFFF, 0x80);

        cpu.run_cycles(7);
        assert_eq!(cpu.read_u16(0x01fe), 0x8003);  
        assert_eq!(cpu.read(0x01fd), (Flags::I | Flags::B | Flags::U).bits());      
        assert_eq!(cpu.stk_ptr, 0xfc);
        assert_eq!(cpu.pc, 0x8004);
        assert!(cpu.status.contains(Flags::I));

        // run rti
        cpu.run_cycles(6);
        assert_eq!(cpu.pc, 0x8003);
        assert_eq!(cpu.stk_ptr, 0xff);



    }

}