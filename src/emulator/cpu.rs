use crate::emulator::bus::{self, Bus};

pub use crate::emulator::instruction::OPCODE_MAP;
pub use crate::emulator::instruction::AddressingMode;


bitflags! {
    //  7 6 5 4 3 2 1 0
    //  N V _ B D I Z C
    //  | |   | | | | +--- Carry Flag
    //  | |   | | | +----- Zero Flag
    //  | |   | | +------- Interrupt Disable
    //  | |   | +--------- Decimal Mode (not used on NES)
    //  | |   +----------- Break Command
    //  | |
    //  | +--------------- Overflow Flag
    //  +----------------- Negative Flag
    //
    pub struct Flags: u8 {
        const NONE = 0;
        const C = (1 << 0);     // Carry Bit
        const Z = (1 << 1);     // Zero
        const I = (1 << 2);     // Disable Interrupts
        const D = (1 << 3);     // Decimal mode (currently unimplemented)
        const B = (1 << 4);     // Break
        const U = (1 << 5);     // Unusued (always 1)
        const V = (1 << 6);     // Overflow
        const N = (1 << 7);     // Negative
    }
}

pub struct CPU6502{
    /* CPU registers */
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    pub stk_ptr: u8,
    pub pc : u16,
    pub status: Flags,

    // =============================
    /* Private internals */
    bus : Box<bus::Bus>,

    // cycles left before instruction completed
    cycles : u32,  

    total_cycles : u32,
    
    // The retrieved/fetched byte
    // operand could also be in
    // addr_rel or addr_abs
    operand : Option<u8>,   

    // address in absolute terms
    // set by (most) address modes
    // and refers to addr of operand
    // or is the actual operand
    addr_abs : Option<u16>,

    // addr change used for REL and branching
    addr_rel : Option<u8>,

    // mode of current operation
    mode : AddressingMode,

    /* Used to keep track incase an additional 
    *  cycle is needed
    *  can be set in addressing modes: ABX, ABY, IDY
    */
    page_crossed : bool,

    // =============================
}

impl CPU6502{
    pub const STACK_OFFSET : u16 = 0x0100;
    pub const RESET_CYCLES: u32 = 8;
    pub const IRQ_CYCLES: u32 = 7;
    pub const NMI_CYCLES: u32 = 8;

    pub fn new(bus : Box<bus::Bus>) -> CPU6502{
        CPU6502{
            reg_a : 0x00,
            reg_x: 0x00,
            reg_y: 0x00,
            stk_ptr: 0xFD,
            pc : 0x0000,
            status: Flags::U, // unused always set

            bus,
            cycles : 0, 
            total_cycles: 0,

            operand : None,
            addr_abs : None, 
            addr_rel : None, 
            mode : AddressingMode::IMP,
            page_crossed : false,      
            
        }
    }

    /// Creates bus and cpu with pc specified
    ///
    /// Internals specified for GUI usage
    pub fn create_cpu_and_bus(pc: u16) -> Self {
        Self{
            reg_a : 0x00,
            reg_x: 0x00,
            reg_y: 0x00,
            stk_ptr: 0xFF,
            pc : pc,
            status: Flags::U, // unused always set

            bus: Box::new(Bus::new()),
            cycles : 0, 
            total_cycles : 0,

            operand : None,
            addr_abs : None, 
            addr_rel : None, 
            mode : AddressingMode::IMP,
            page_crossed : false,      
            
        }
    }


    /// default load that calls reset
    /// this means you must execute 8 clock cycles
    /// before it can complete the next instruction
    pub fn load(&mut self, program : Vec<u8>) -> Result<(), &'static str> {
        self.bus.load_cpu(program, None)?;

        // if load was succsessful, wrote 0x8000
        // at 0xFFFC
        self.reset();

        Ok(())
    }

    /// Load starting at a specified address
    /// Also calls reset, meaning execute 8 clocks before running
    pub fn specific_load(&mut self, program : Vec<u8>, start_addr: u16) -> Result<(), &'static str> {
        self.bus.load_cpu(program, Some(start_addr))
    }

    /// Calls clock the specified number of times
    pub fn run_cycles(&mut self, cycles : u32){
        for _i in 1..=cycles {
            self.clock();
        }
        // total cycles adjusted with clock
    }

    /// Returns the number of total cycles ran on the cpu instance/
    /// 
    /// Affected by clock(),
    /// , run_cycles(), and run_until_brk()
    pub fn get_total_cycles(&self) -> u32 {
        self.total_cycles
    }

    /// Returns data at address, handled by bus implementation
    pub fn read(&self, addr: u16) -> u8 {
        self.bus.read(addr)
    }

    /// Returns two bytes combined, following little endian
    pub fn read_u16(&self, addr: u16) -> u16{
        let lo = self.bus.read(addr);
        let hi = self.bus.read(addr.wrapping_add(1));

        (hi as u16) << 8 | lo as u16
    }

    /// Writes to address, handled by bus implementation
    pub fn write(&mut self, addr: u16, val : u8) {
        self.bus.write(addr, val);
    }

    // incriments pc by 1
    fn read_pc(&mut self) -> u8 {
        let val = self.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        val
    }

    // incriments pc by 2
    fn read_pc_u16(&mut self) -> u16{
        let lo = self.read_pc() as u16;
        let hi = self.read_pc() as u16;

        hi << 8 | lo

    }

    /// Indexes from start..=end
    pub fn index_memory(&self, start: u16, end: u16) -> Option<&[u8]> { 
        self.bus.index_memory(start, end)
    }

    /// Will treat unknown opcodes as NOPs
    /// aka designed for GUI usage
    pub fn run_until_brk(&mut self) {
        let map = &*OPCODE_MAP;

        let mut opcode = self.read_pc();
        while opcode!= 0x00 {

            // opcode in unwrap is that of NOP
            let instr = map.get(&opcode).unwrap_or(map.get(&0xEA).unwrap());

            let mode = instr.mode; 

            self.run_addr_mode(mode);
            self.run_operation(instr.opcode, mode);

            // set internal variables to none after operation is complete 
            self.operand = None;
            self.addr_abs = None; 
            self.addr_rel = None;

            self.total_cycles += self.cycles;
            opcode = self.read_pc();

        }
    }

    /// Ignores clock cycles and exectues
    /// the next instruction
    ///
    /// Returns number of cycles instruction took
    /// 
    /// Treats invalid instructions as NOPs
    /// aka designed for GUI usage
    pub fn execute_step(&mut self) -> u32 {
        // fetch opcode
        let opcode = self.read_pc();

        let map = &*OPCODE_MAP;

        // NOP opcode in unwrap
        let instr = map.get(&opcode).unwrap_or(OPCODE_MAP.get(&0xEA).unwrap());

        let mode = instr.mode;

        self.run_addr_mode(mode);
        self.run_operation(instr.opcode, mode);

        // set internal variables to none after operation is complete 
        self.operand = None;
        self.addr_abs = None; 
        self.addr_rel = None;
        
        self.total_cycles += self.cycles;
        self.cycles
    }

    /// If clock cycle is 0, runs an instruction and appropriately sets internal cycles.
    /// Will always decrement internal cycle count.
    /// 
    /// If an instruction takes 5 cycles, will have to call clock 5 times before
    /// the cpu can run the next instruction
    pub fn clock(&mut self) {

        if self.cycles == 0 {
            // fetch opcode
            let opcode = self.read_pc();

            let map = &*OPCODE_MAP;

            let instr = map.get(&opcode).unwrap_or_else
                (|| {panic!("Opcode{} not found in instr map!\n", opcode)}
                );

            self.cycles = instr.min_cycles as u32;
            let mode = instr.mode;

            self.run_addr_mode(mode);
            self.run_operation(opcode, mode);

            // set internal variables to none after operation is complete 
            self.operand = None;
            self.addr_abs = None; 
            self.addr_rel = None;
        }

        self.cycles -= 1;
        self.total_cycles += 1;
    }

    /// Must run appropriate amoutn of cycles to allow cpu to continue after
    /// calling this
    pub fn reset(&mut self) {

        // set registers
        self.reg_a = 0;
        self.reg_x = 0;
        self.reg_y = 0;

        self.stk_ptr = 0xFD;
        
        // Unused bit set (and interupts enabled)
        self.status = Flags::U;

        // Read vector according to 6502 specification 
        self.pc = self.read_u16(0xFFFC);

        self.cycles = CPU6502::RESET_CYCLES;

        // Clean private internals
        self.operand = None;
        self.addr_abs = None;
        self.addr_rel = None;
        self.mode = AddressingMode::IMP;
    }

    /// Interrupt Request
    ///  * Only runs if interrupts enabled (Flags::I == 0)
    ///
    /// Must run appropriate amoutn of cycles to allow cpu to continue after
    /// calling this
    pub fn irq(&mut self) {

        if !self.status.contains(Flags::I){
            // push pc into stack
            //  follow little endian by pushing
            //  high byte first
            self.stack_push((self.pc >> 8) as u8); 
            self.stack_push((self.pc & 0x00FF) as u8);

            // push status after indicating interrupt
            // has occured
            self.status.insert(Flags::I | Flags::U);
            self.status.remove(Flags::B);
            self.stack_push(self.status.bits());

            // Read vector according to 6502 specification 
            self.pc = self.read_u16(0xFFFE);

            self.cycles = CPU6502::IRQ_CYCLES;

            // Clean private internals
            self.operand = None;
            self.addr_abs = None;
            self.addr_rel = None;
            self.mode = AddressingMode::IMP;
        }
        
    }

    /// Non-maskable interrupt
    ///
    /// Must run appropriate amoutn of cycles to allow cpu to continue after
    /// calling this
    pub fn nmi(&mut self)  {
        // push pc into stack
        //  follow little endian by pushing
        //  high byte first
        self.stack_push((self.pc >> 8) as u8); 
        self.stack_push((self.pc & 0x00FF) as u8);

        // push status after indicating interrupt
        // has occured
        self.status.insert(Flags::I | Flags::U);
        self.status.remove(Flags::B);
        self.stack_push(self.status.bits());

        // Read vector according to 6502 specification 
        self.pc = self.read_u16(0xFFFA);

        self.cycles = CPU6502::NMI_CYCLES;

        // Clean private internals
        self.operand = None;
        self.addr_abs = None;
        self.addr_rel = None;
        self.mode = AddressingMode::IMP;
    }

    // sets up internals (addr_abs, addr_rel, page_crossed) 
    // and pc appropriately 
    fn run_addr_mode(&mut self, mode : AddressingMode) {
        use AddressingMode::*;

        // set at the beginning
        self.page_crossed = false;

        match mode {

            IMM => {
                self.addr_abs = Some(self.pc); 
                self.pc += 1;
            }

            ACC => {
                // doesn't use addr
                self.operand = Some(self.reg_a);
            }

            REL => {

                self.addr_rel = Some(self.read_pc());

            }

            ZP0 => {
                self.addr_abs = Some(self.read_pc() as u16);
            }

            ZPX => {
                let byte = self.read_pc();
                self.addr_abs = Some(byte.wrapping_add(self.reg_x) as u16);

            }

            ZPY => {
                let byte = self.read_pc();
                self.addr_abs = Some(byte.wrapping_add(self.reg_y) as u16);
                
            }  

            ABS => {
                self.addr_abs = Some(self.read_pc_u16());
            }

            ABX => {
                let read_addr = self.read_pc_u16();
                let new_addr = read_addr.wrapping_add(self.reg_x as u16);

                self.page_crossed = (new_addr & 0xFF00) != (read_addr & 0xFF00);

                self.addr_abs = Some(new_addr);

            }

            ABY => {
                let read_addr = self.read_pc_u16();
                let new_addr = read_addr.wrapping_add(self.reg_y as u16);

                self.page_crossed = (new_addr & 0xFF00) != (read_addr & 0xFF00);

                self.addr_abs = Some(new_addr);
            }

            IND => {  
                let base_ptr = self.read_pc_u16();

                /* 
                * Also see: http://obelisk.me.uk/6502/reference.html#JMP
                *
                * bug example: 
                *   base_ptr = 0x10FF
                *   0x10FF has 40 stored, 0x1100 -> 50, 0x1000 -> 60
                *   result ideally would be 0x5040, but we get 0x6040
                *   => next_ptr = 0x1000
                */

                // simulating page boundary hardware bug
                let next_ptr = match base_ptr & 0x00FF {

                    // need to cross a page boundary to reach next_ptr
                    0x00FF => base_ptr & 0xFF00,

                    _ => base_ptr + 1
                };

                let lo = self.read(base_ptr) as u16;
                let hi = self.read(next_ptr) as u16;
                
                self.addr_abs = Some((hi << 8) | lo);
            }

            IDX => {
                /* supplied 8-bit address is offset by x to index
                *  a location in page 0x00. Operand's 16 bit address
                *  is stored in this location
                */ 
                let ptr = self.read_pc().wrapping_add(self.reg_x);
                let nxt = ptr.wrapping_add(1);

                let lo = self.read(ptr as u16) as u16;
                let hi = self.read(nxt as u16) as u16;

                self.addr_abs = Some((hi << 8) | lo);
            }

            IDY => {

                /* supplied 8-bit address indexs in page 0x00.
                *  The 16 bit address retrieved is then offset
                *  by y to get the operand's address.
                */

                let ptr = self.read_pc();
                let nxt = ptr.wrapping_add(1);

                let lo = self.read(ptr as u16) as u16;
                let hi = self.read(nxt as u16) as u16;

                let retrieved_addr = (hi << 8) | lo; // let addr = self.read_u16(ptr as u16);
                let op_addr = retrieved_addr.wrapping_add(self.reg_y as u16);

                self.page_crossed = (op_addr & 0xFF00) != (retrieved_addr & 0xFF00);

                self.addr_abs = Some(op_addr);

            }

            IMP => {
                // nothing happens
            }
            
        };
    }
    
    // run_addr_mode before hand being ran to set internals properly
    fn run_operation(&mut self, opcode : u8, mode : AddressingMode){

        self.mode = mode;

        // set-up operand using addr_abs if needed 
        self.operand = match mode{
            /* operand doesn't need to be set */
            AddressingMode::IMP => None,

            // Need addr_rel set
            AddressingMode::REL => None,

            // Need only addr_abs set
            AddressingMode::IND => None,

            // operand should have already been set to a
            // but do it again just in case  
            AddressingMode::ACC => Some(self.reg_a),

            // need to fetch operand from specified addr
            _ => 
                Some(self.read(self.addr_abs.unwrap()))

        };

        match opcode{

            0x00 => {
                self.brk();
            }

            0xEA => {
                self.nop();
            }

            /* Unofficial unimplemented */ 
            0x1C | 0x3C | 0x5C | 0xDC | 0xFC => {
                self.unimplemented_unofficial();
            }

            /* ADC */
            0x69 | 0x65 | 0x75 | 0x6d | 0x7d | 0x79 | 0x61 | 0x71 => {
                self.adc();
            }

            /* AND */ 
            0x29 | 0x25 | 0x35 | 0x2d | 0x3d | 0x39 | 0x21 | 0x31 => {
                self.and();
            }
            
            /* ASL */
            0x0a | 0x06 | 0x16 | 0x0e | 0x1e => {
                self.asl();
            }

            /* BCC */
            0x90 => {
                self.bcc();
            }

            /* BCS */
            0xB0 => {
                self.bcs();
            }

            /* BEQ */
            0xf0 => {
                self.beq();
            }

            /* BMI */
            0x30 => {
                self.bmi();
            }

            /* BNE */
            0xd0 => {
                self.bne();
            }

            /* BPL */
            0x10 => {
                self.bpl();
            }

            /* BVC */
            0x50 => {
                self.bvc();
            }

            /* BVS */
            0x70 => {
                self.bvs();
            }

            /* BIT */
            0x24 | 0x2c => {
                self.bit();
            }

            /* CLC */
            0x18 => {
                self.clc();
            }

            /* CLD */
            0xD8 => {
                self.cld();
            }

            /* CLI */
            0x58 => {
                self.cli();
            }

            /* CLV */
            0xB8 => {
                self.clv();
            }

            /* CMP */
            0xc9 | 0xc5 | 0xd5 | 0xcd | 0xdd | 0xd9 | 0xc1 | 0xd1  => {
                self.cmp();
            }

            /* CPX */
            0xe0 | 0xe4 | 0xec => {
                self.cpx();
            }

            /* CPY */
            0xc0 | 0xc4 | 0xcc => {
                self.cpy();
            }

            /* DEC */
            0xc6 | 0xd6 | 0xce | 0xde => {
                self.dec();
            }

            /* DEX */
            0xca => {
                self.dex();
            }

            /* DEY */
            0x88 => {
                self.dey();
            }

            /* EOR */
            0x49 | 0x45 | 0x55 | 0x4d | 0x5d | 0x59 | 0x41 | 0x51 => {
                self.eor();
            }
            
            /* INC */
            0xe6 | 0xf6 | 0xee | 0xfe => {
                self.inc();
            }

            /* INX */
            0xe8 => {
                self.inx();
            }

            /* IDY */
            0xc8 => {
                self.iny();
            }

            /* JMP */
            0x4c | 0x6c => {
                self.jmp();
            }
            
            /* JSR */
            0x20 => {
                self.jsr();
            }

            /* LDA */
            0xa9 | 0xa5 | 0xb5 | 0xad | 0xbd | 0xb9 | 0xa1 | 0xb1 => {
                self.lda();
            }

            /* LDX */
            0xa2 | 0xa6 | 0xb6 | 0xae | 0xbe => {
                self.ldx();
            }

            /* LDY */
            0xa0 | 0xa4 | 0xb4 | 0xac | 0xbc => {
                self.ldy();
            }

            /* LSR */
            0x4a| 0x46 | 0x56 | 0x4e | 0x5e => {
                self.lsr();
            }

            /* ORA */
            0x09 | 0x05 | 0x15 | 0x0d | 0x1d | 0x19 | 0x01 | 0x11 => {
                self.ora();
            }

            /* PHA */
            0x48  => {
                self.pha();
            }

            /* PHP */
            0x08  => {
                self.php();
            }

            /* PLA */
            0x68  => {
                self.pla();
            }

            /* PLP */
            0x28  => {
                self.plp();
            }

            /* ROL */
            0x2a | 0x26 | 0x36 | 0x2e | 0x3e => {
                self.rol();
            }

            /* ROR */
            0x6a | 0x66 | 0x76 | 0x6e | 0x7e => {
                self.ror();
            }

            /* RTI */
            0x40 => {
                self.rti();
            }

            /* RTS */
            0x60 => {
                self.rts();
            }

            /* SBC */
            0xe9 | 0xe5 | 0xf5 | 0xed | 0xfd | 0xf9 | 0xe1 | 0xf1 => {
                self.sbc();
            }

            /* SEC */
            0x38 => {
                self.sec();
            }

            /* SED */
            0xf8 => {
                self.sed();
            }

            /* SEI */
            0x78 => {
                self.sei();
            }

            /* STA */
            0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91 => {
                self.sta();
            }

            /* STX */
            0x86 | 0x96 | 0x8e => {
                self.stx();
            }

            /* STY */
            0x84 | 0x94 | 0x8c => {
                self.sty();
            }
            
            /* TAX */
            0xAA => {
                self.tax();
            }

            /* TAY */
            0xA8 => {
                self.tay();
            }

            /* TSX */
            0xBA => {
                self.tsx();
            }

            /* TXA */
            0x8A => {
                self.txa();
            }

            /* TXS */
            0x9A => {
                self.txs();
            }

            /* TYA */
            0x98 => {
                self.tya();
            }

            _ => { 
                panic!("Unhandled Opcode: {}!!! ", opcode)
            }
        }

    }

    /* Internal functions*/
    fn branch(&mut self){
        self.cycles += 1;

        let delta = self.addr_rel.unwrap() as i8;

        let addr = self.pc.wrapping_add(delta as u16);

        if self.pc & 0xFF00 != addr & 0xFF00 {
            self.cycles += 1;
        }

        self.pc = addr;

    }

    fn compare(&mut self, reg : u8) {
        if self.page_crossed {
            self.cycles += 1;
        }

        let operand = self.operand.unwrap();

        self.status.set(Flags::C, reg >= operand);

        let tmp = reg.wrapping_sub(operand);

        self.status.set(Flags::Z, tmp == 0);
        self.status.set(Flags::N, tmp &0x80 != 0);
    }

    fn stack_push(&mut self, data : u8) {
        let addr = self.stk_ptr as u16 + CPU6502::STACK_OFFSET;
        self.write(addr, data);
        self.stk_ptr = self.stk_ptr.wrapping_sub(1);
    }

    fn stack_pop(&mut self) -> u8 {
        self.stk_ptr = self.stk_ptr.wrapping_add(1);
        let addr = self.stk_ptr as u16 + CPU6502::STACK_OFFSET;
        self.read(addr)
    }


    /* Operations to run */
    //  Only need to account for extra cycles, default cycless
    //  handled in run_operation()

    fn brk(&mut self) {

        /* must push pc+2, meaning if code looks like this: 
        *  BRK $2000 $2001
        *  stored pc must be 2001
        *  specified here: https://www.masswerk.at/6502/6502_instruction_set.html#BRK
        */
        let pc = self.pc.wrapping_add(1); // have already added 1 when reading BRK's opcode

        // push hi, then lo
        self.stack_push((pc>>8) as u8);
        self.stack_push((pc & 0x00FF) as u8);

        let stored_status = self.status | Flags::I | Flags::B | Flags::U;
        self.stack_push(stored_status.bits());

        // following specifications, read new pc from 0xFFFE at brk
        self.pc = self.read_u16(0xFFFE); 
        
        self.status.set(Flags::I, true);
        self.status.set(Flags::B, false);
    }

    fn nop(&mut self) {
        // does nothing, pc has already been incrimented
        // to read NOP opcode
    }

    fn unimplemented_unofficial(&self) {
        panic!("Running unoffical opcode ({:02x}) not yet supported!", self.read(self.pc - 1));
    }


    fn adc(&mut self) {
        // (see _1_ in instruction.rs)
        if self.page_crossed {
            self.cycles += 1;
        }
        
        let operand = self.operand.unwrap();

        let mut result = self.reg_a as u16;
        result += operand as u16;

        if self.status.contains(Flags::C){
            result += 1;
        }

        // value greater than max 8 bit number
        self.status.set(Flags::C, result > 0xFF);
        
        // if 8 bit version is 0
        self.status.set(Flags::Z, (result & 0x00FF) == 0);

        // bit 7 is 1 => negative in signed
        self.status.set(Flags::N, result & 0x0080 != 0);

        // removes bits higher than bit 7
        let r = result as u8;

        /* see: https://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
        *
        * P denote a positive signed 8 bit numbers
        * while N denotes a negative. Overflow happens
        * in the following cases:
        *   if P1 + P2 = N => overflow
        *   if N1 + N2 = P => overflow
        * 
        * Meaning overflow happens if two
        * operand's MSBs are the opposite
        * of result's MSB
        */
        let v = (self.reg_a ^ r) & (operand ^ r) & 0x80;
        let overflow = v != 0;
        self.status.set(Flags::V, overflow);

        self.reg_a = r;


    }

    fn and(&mut self) {
        let operand = self.operand.unwrap();

        let a = self.reg_a & operand;

        self.status.set(Flags::Z, a == 0);
        self.status.set(Flags::N, a & 0x80 != 0);

        self.reg_a = a;

        if self.page_crossed {
            self.cycles += 1;
        }

    }

    fn asl(&mut self) {
        let operand = self.operand.unwrap();
        let tmp = operand << 1;

        // original bit7 placed in carry bit
        self.status.set(Flags::C, operand & 0x80 != 0);

        // check bit7 after shift
        self.status.set(Flags::N, tmp & 0x80 != 0);

        self.status.set(Flags::Z, tmp == 0);
        

        match self.mode {

            AddressingMode::ACC => {
                self.reg_a = tmp;
            }
            
            AddressingMode::ZP0 | AddressingMode::ZPX |
            AddressingMode::ABS | AddressingMode::ABX 
            => {
                self.write(self.addr_abs.unwrap(), tmp);
            }

            _ => panic!("Incorrect Addressing Mode in ASL\n")
        };

    }

    fn bcc(&mut self) {
        if !self.status.contains(Flags::C){
            self.branch();
        }
    }

    fn bcs(&mut self) {
        if self.status.contains(Flags::C){
            self.branch();
        }
    }

    fn beq(&mut self) {
        if self.status.contains(Flags::Z){
            self.branch();
        }
    }

    fn bmi(&mut self) {
        if self.status.contains(Flags::N){
            self.branch();
        }
    }

    fn bne(&mut self) {
        if !self.status.contains(Flags::Z){
            self.branch();
        }
    }

    fn bpl(&mut self) {
        if !self.status.contains(Flags::N){
            self.branch();
        } 
    }

    fn bvc(&mut self) {
        if !self.status.contains(Flags::V){
            self.branch();
        }
    }

    fn bvs(&mut self) {
        if self.status.contains(Flags::V){
            self.branch();
        }
    }

    fn bit(&mut self) {
        let tst = self.operand.unwrap() & self.reg_a;

        self.status.set(Flags::Z, tst == 0);
        // 6th bit sets V
        self.status.set(Flags::V, tst & 0x40 != 0);
        // 7th but sets N
        self.status.set(Flags::N, tst & 0x80 != 0);

    }

    fn clc(&mut self) {

        self.status.remove(Flags::C);

    }

    fn cld(&mut self) {
        
        self.status.remove(Flags::D);

    }

    fn cli(&mut self) {
        
        self.status.remove(Flags::I);

    }
    
    fn clv(&mut self) {

        self.status.remove(Flags::V);

    }

    fn cmp(&mut self) {
        self.compare(self.reg_a);
    }

    fn cpx(&mut self) {
        self.compare(self.reg_x);
    }

    fn cpy(&mut self) {
        self.compare(self.reg_y);
    }

    fn dec(&mut self) {
        let val = self.operand.unwrap().wrapping_sub(1);
        
        self.status.set(Flags::Z, val == 0);
        self.status.set(Flags::N, val & 0x80 != 0);

        self.write(self.addr_abs.unwrap(), val);

    }

    fn dex(&mut self) {
        let val = self.reg_x.wrapping_sub(1);

        self.status.set(Flags::Z, val == 0);
        self.status.set(Flags::N, val & 0x80 != 0);

        self.reg_x = val;
    }

    fn dey(&mut self) {
        let val = self.reg_x.wrapping_sub(1);

        self.status.set(Flags::Z, val == 0);
        self.status.set(Flags::N, val & 0x80 != 0);

        self.reg_y= val;
    }

    fn eor(&mut self) {
        self.reg_a ^= self.operand.unwrap();

        self.status.set(Flags::Z, self.reg_a == 0);
        self.status.set(Flags::N, self.reg_a & 0x80 != 0);

        if self.page_crossed {
            self.cycles += 1;
        }
    }

    fn inc(&mut self) {
        let val = self.operand.unwrap().wrapping_add(1);
        
        self.status.set(Flags::Z, val == 0);
        self.status.set(Flags::N, val & 0x80 != 0);

        self.write(self.addr_abs.unwrap(), val);
    }

    fn inx(&mut self) {
        let val = self.reg_x.wrapping_add(1);

        self.status.set(Flags::Z, val == 0);
        self.status.set(Flags::N, val & 0x80 != 0);

        self.reg_x = val;
    }

    fn iny(&mut self) {
        let val = self.reg_x.wrapping_add(1);

        self.status.set(Flags::Z, val == 0);
        self.status.set(Flags::N, val & 0x80 != 0);

        self.reg_y = val;
    }

    fn jmp(&mut self) {
        self.pc = self.addr_abs.unwrap();
    }

    fn jsr(&mut self) {
        /*
        * https://www.masswerk.at/6502/6502_instruction_set.html#JSR
        * if JSR opcode located at 6000,
        * we will push in 6002 to the stack.
        * 
        * After running abs addr mode, pc
        * is on next opcode (6003), so 
        * we need to subtract 1
        */

        let ret = self.pc.wrapping_sub(1);
        // push return addr into stack
        self.stack_push((ret >> 8) as u8); 
        self.stack_push((ret & 0x00FF) as u8);

        self.pc = self.addr_abs.unwrap();

    }

    fn lda(&mut self) {
        self.reg_a = self.operand.unwrap();

        self.status.set(Flags::Z, self.reg_a == 0);
        self.status.set(Flags::N, self.reg_a & 0x80 != 0);

        if self.page_crossed {
            self.cycles += 1;
        }
    }

    fn ldx(&mut self) {
        self.reg_x = self.operand.unwrap();

        self.status.set(Flags::Z, self.reg_x == 0);
        self.status.set(Flags::N, self.reg_x & 0x80 != 0);

        if self.page_crossed {
            self.cycles += 1;
        }
    }

    fn ldy(&mut self) {
        self.reg_y = self.operand.unwrap();

        self.status.set(Flags::Z, self.reg_y == 0);
        self.status.set(Flags::N, self.reg_y & 0x80 != 0);

        if self.page_crossed {
            self.cycles += 1;
        }
    }

    fn lsr(&mut self) {
        let operand = self.operand.unwrap();
        let tmp = operand >> 1;

        // original bit 0 placed in carry bit
        self.status.set(Flags::C, operand & 0x01 != 0);

        // check bit7 after shift
        self.status.set(Flags::N, tmp & 0x80 != 0);

        self.status.set(Flags::Z, tmp == 0);
        

        match self.mode {

            AddressingMode::ACC => {
                self.reg_a = tmp;
            }
            
            AddressingMode::ZP0 | AddressingMode::ZPX |
            AddressingMode::ABS | AddressingMode::ABX 
            => {
                self.write(self.addr_abs.unwrap(), tmp);
            }

            _ => panic!("Incorrect Addressing Mode in LSR\n")
        };
    }

    fn ora(&mut self) {
        self.reg_a |= self.operand.unwrap();

        self.status.set(Flags::Z, self.reg_a == 0);
        self.status.set(Flags::N, self.reg_a & 0x80 != 0);

        if self.page_crossed {
            self.cycles += 1;
        }
    }
    
    fn pha(&mut self) {
        self.stack_push(self.reg_a);
    }

    fn php(&mut self) {
        self.stack_push(self.status.bits());
    }

    fn pla(&mut self) {

        self.reg_a = self.stack_pop();

        self.status.set(Flags::Z, self.reg_a == 0);
        self.status.set(Flags::N, self.reg_a & 0x80 != 0);
    }

    fn plp(&mut self) {
        self.status = Flags::from_bits_truncate(self.stack_pop());
    }

    fn rol(&mut self) {
        let operand = self.operand.unwrap();
        let mut tmp = operand << 1;

        tmp |= match self.status.contains(Flags::C) {
            true => 0x01,
            false => 0x00
        };


        // original bit 0 placed in carry bit
        self.status.set(Flags::C, operand & 0x01 != 0);

        // check bit7 after shift
        self.status.set(Flags::N, tmp & 0x80 != 0);

        self.status.set(Flags::Z, tmp == 0);
        

        match self.mode {

            AddressingMode::ACC => {
                self.reg_a = tmp;
            }
            
            AddressingMode::ZP0 | AddressingMode::ZPX |
            AddressingMode::ABS | AddressingMode::ABX 
            => {
                self.write(self.addr_abs.unwrap(), tmp);
            }

            _ => panic!("Incorrect Addressing Mode in ROL\n")
        };
    } 
    

    fn ror(&mut self) {
        let operand = self.operand.unwrap();
        let mut tmp = operand >> 1;

        tmp |= match self.status.contains(Flags::C) {
            true => 0x80,
            false => 0x00
        };


        // original bit 0 placed in carry bit
        self.status.set(Flags::C, operand & 0x01 != 0);

        // check bit7 after shift
        self.status.set(Flags::N, tmp & 0x80 != 0);

        self.status.set(Flags::Z, tmp == 0);
        

        match self.mode {

            AddressingMode::ACC => {
                self.reg_a = tmp;
            }
            
            AddressingMode::ZP0 | AddressingMode::ZPX |
            AddressingMode::ABS | AddressingMode::ABX 
            => {
                self.write(self.addr_abs.unwrap(), tmp);
            }

            _ => panic!("Incorrect Addressing Mode in ROR\n")
        };
    }

    fn rti(&mut self) {
        let mut flags = Flags::from_bits_truncate(self.stack_pop());

        // before placing it back in reg, double ensuring B is 0
        flags.remove(Flags::B);
        flags.insert(Flags::U);

        self.status = flags;


        self.pc = self.stack_pop() as u16;
        self.pc |= (self.stack_pop() as u16) << 8;


    }

    fn rts(&mut self) {
        /*see: https://www.masswerk.at/6502/6502_instruction_set.html#RTS */

        // pulls value from stack
        let mut ret = self.stack_pop() as u16;
        ret |= (self.stack_pop() as u16) << 8;

        // as jsr pushes the address before the next
        // opcode, we must incriment pc
        self.pc = ret.wrapping_add(1);
    }

    // There is no way to subtract without the carry which works as an inverse borrow. 
    // i.e, to subtract you set the carry before the operation.
    // If the carry is cleared by the operation, it indicates a borrow occurred.
    fn sbc(&mut self) {
        // another explanation: https://en.wikipedia.org/wiki/Carry_flag#Vs._borrow_flag
        let tmp = self.operand.unwrap();
        /*
        * Note:
        *    The carry bit not being set means
        *    that a 1 was "borrowed" in a previous
        *    subtraction, while if it was set that 
        *    means the subtraction has not had a borrow
        *
        *   -1*(M) = !(M) + 1 (in binary)
        *       => -M - 1 = !M
        *   Using this we, know for subtraction
        *   to work considering only a single byte
        *   carry bit must be 1 so we 
        *   may add to offset the -1 (equivalent to no borrow)
        *   
        *   
        */
        self.operand = Some(!tmp);
        self.adc();

        self.operand = Some(tmp);
    }

    fn sec(&mut self) {
        self.status.insert(Flags::C);
    }

    fn sed(&mut self) {
        self.status.insert(Flags::D);
    }

    fn sei(&mut self) {
        self.status.insert(Flags::I);
    }

    fn sta(&mut self) {
        self.write(self.addr_abs.unwrap(), self.reg_a);
    }

    fn stx(&mut self) {
        self.write(self.addr_abs.unwrap(), self.reg_x);
    }

    fn sty(&mut self) {
        self.write(self.addr_abs.unwrap(), self.reg_y);
    }

    fn tax(&mut self) {
        self.reg_x = self.reg_a;

        self.status.set(Flags::Z, self.reg_x == 0);
        self.status.set(Flags::N, self.reg_x & 0x80 != 0);
    }

    fn tay(&mut self) {
        self.reg_y = self.reg_a;

        self.status.set(Flags::Z, self.reg_y == 0);
        self.status.set(Flags::N, self.reg_y & 0x80 != 0);
    }

    fn tsx(&mut self) {
        self.reg_x = self.stk_ptr;

        self.status.set(Flags::Z, self.reg_x == 0);
        self.status.set(Flags::N, self.reg_x & 0x80 != 0);
    }

    fn txa(&mut self) {
        self.reg_a = self.reg_x;

        self.status.set(Flags::Z, self.reg_a == 0);
        self.status.set(Flags::N, self.reg_a & 0x80 != 0);
    }

    fn txs(&mut self) {
        self.stk_ptr = self.reg_x;
    }

    fn tya(&mut self) {
        self.reg_a = self.reg_y;

        self.status.set(Flags::Z, self.reg_a == 0);
        self.status.set(Flags::N, self.reg_a & 0x80 != 0);        
    }


}



