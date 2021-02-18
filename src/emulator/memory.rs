use std::usize;

const CPU_RAM_SIZE : usize = 64*1024; // 64 KB


#[allow(non_camel_case_types)]
pub struct CPU_RAM(Box<[u8; CPU_RAM_SIZE]>);

pub trait Memory {
    fn read(&self, addr:u16) -> u8;

    fn write(&mut self, addr:u16, val:u8);

    fn read_u16(&self, addr:u16) -> u16;

    fn write_u16(&mut self, addr:u16, val:u16);

    fn load(&mut self, program : Vec<u8>, start_addr : Option<u16>) -> Result<(), &'static str>;
}
/*  CPU Memory Map
*   
* 0xFFFF -> ===================
*           program ROM
*
* 0x8000 -> -------------------
*           save RAM
* 0x6000 -> -------------------
*           expansion ROM
* 0x4020 -> -------------------
*           IO Registers
*
* 0x2000 -> -------------------
*           CPU RAM
*
* 0x0000 -> ===================
*
*/

impl CPU_RAM {
    pub fn new() -> CPU_RAM{
        CPU_RAM(Box::new([0; CPU_RAM_SIZE]))
    }

    pub fn index_memory(&self, start: u16, end: u16) -> Option<&[u8]>{
        let start = start as usize;
        let end = end as usize;

        self.0.get(start..=end)
    }

}

impl Memory for CPU_RAM{

    fn read(&self, addr:u16) -> u8 {
        self.0[addr as usize]
    }

    fn write(&mut self, addr:u16, val:u8){
        let addr = addr as usize;
        self.0[addr] = val;
    }

    fn read_u16(&self, addr:u16) -> u16 {
        let lo = self.0[addr as usize];
        let hi = self.0[addr.wrapping_add(1) as usize];

        (hi as u16)<< 8 | lo as u16
    }

    fn write_u16(&mut self, addr:u16, val:u16){
        self.0[addr as usize] = val as u8;
        self.0[addr.wrapping_add(1) as usize] = (val >> 8) as u8;

    }

    fn load(&mut self, program : Vec<u8>, start_addr : Option<u16>) -> Result<(), &'static str> {
        // memory for program ROM should be btwn 0x8000 and 0xFFFF

        let start_addr = match start_addr{
            Some(x) => x,
            None => 0x8000,
        };

        if program.len() > 0xFFFC - start_addr as usize{
            return Err("Program len too large")
        }

        // write for reset
        self.write_u16(0xFFFC, start_addr);

        let start_addr = start_addr as usize;

        self.0[start_addr..(start_addr + program.len())].copy_from_slice(&program[..]);
        Ok(())
    }


}



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut ram = CPU_RAM::new();
        let addr = 200;
        let x = 100;
        
        assert_eq!(0, ram.read(addr));
        
        ram.write(addr, x);

        assert_eq!(x, ram.read(addr));
    }
}
