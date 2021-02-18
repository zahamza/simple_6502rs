use crate::emulator::memory::{CPU_RAM, Memory};

/*
*   Will handle memory mirroring here
* 
*/


pub struct Bus{

    cpu_ram : CPU_RAM, // 64KB for now

}


impl Bus{
    pub fn new() -> Bus{
        Bus{
            cpu_ram : CPU_RAM::new(),
        }
    }

    /// Indexes from start..=end
    pub fn index_memory(&self, start: u16, end: u16) -> Option<&[u8]> {
        self.cpu_ram.index_memory(start, end)
    }
    

    pub fn write(&mut self, addr : u16, val : u8) {

        match addr {
            0x0000..=0xFFFF => {
                self.cpu_ram.write(addr, val);
            }  
        }

    } 

    pub fn read(&self, addr: u16) -> u8 {
        match addr{
            0x0000..=0xFFFF => {
                self.cpu_ram.read(addr)
            }

        }
    }

    pub fn load_cpu(&mut self, program: Vec<u8>, start_addr : Option<u16>)
    -> Result<(), &'static str> {
        self.cpu_ram.load(program, start_addr)
    }

}   

