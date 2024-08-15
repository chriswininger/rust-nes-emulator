pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,

    // status holds the flags -- 7 bits ->
    // carry, zero, interrupt disable, decimal mode, break command, overflow flag, negative
    // (these are reverse order)
    // https://www.nesdev.org/obelisk-6502-guide/registers.html#C
    pub status: u8,
    pub program_counter: u16,

    // array to hold 64 KiB of address space (2KiB RAM, rest is reserved for
    // memory mapping
    memory: [u8; 0xFFFF],
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: [0; 0xFFFF]
        }
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run()
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.status = 0;

        // initialize program counter to to byte value stored in 0xFFFC
        self.program_counter  = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        // 0x8000 to 0xFFFF is reserved for the Program ROM
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn run(&mut self) {
        // note: we move initialization of program_counter from here to load
        loop {
            // let opscode = program[self.program_counter as usize];
            let opscode = self.mem_read(self.program_counter);
            self.program_counter += 1;

            println!("!!! running opscode: {:x}", opscode);
             // opcodes https://www.nesdev.org/obelisk-6502-guide/reference.html
            match opscode {
                0xA9 => { // LDA
                    let param =  self.mem_read(self.program_counter);
                    self.program_counter += 1;

                    self.lda(param);
                }
                0xAA => self.tax(),
                // I imeplemented this thinking it was asked for by the tutorial but really
                // they were loading the value of 0xc0, this may or may not be working :-)
                0xc0 => { // CPY - Compare Y Register
                    let param = self.mem_read(self.program_counter);
                    self.program_counter += 1;

                    if self.register_y > param {
                        // set carry flag
                        self.set_carry_flag();
                    } else if self.register_y == param {
                        // set 0 flag
                        self.status = self.status | 0b0000_0010;
                    }

                }
                0xe8 => { // INX - Increment X Register
                    println!("register_x: {}", self.register_x);

                    if self.register_x < 0xff {
                        self.register_x = self.register_x + 1;
                    } else {
                        self.register_x = 0x00;
                    }

                    println!("register_x + 1: {}", self.register_x);

                    self.update_zero_and_negative_flags(self.register_x);
                }
                0x00 => return, // BRK
                _ => todo!()
            }
            
        }
    }

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.program_counter = 0;

        loop {
            let opscode = program[self.program_counter as usize];
            self.program_counter += 1;

            // opcodes https://www.nesdev.org/obelisk-6502-guide/reference.html
            match opscode {
                0xA9 => { // LDA
                    let param = program[self.program_counter as usize];
                    self.program_counter += 1;

                    self.lda(param);
                }
                0xAA => self.tax(),
                // I imeplemented this thinking it was asked for by the tutorial but really
                // they were loading the value of 0xc0, this may or may not be working :-)
                0xc0 => { // CPY - Compare Y Register
                    let param = program[self.program_counter as usize];
                    self.program_counter += 1;

                    if self.register_y > param {
                        // set carry flag
                        self.set_carry_flag();
                    } else if self.register_y == param {
                        // set 0 flag
                        self.status = self.status | 0b0000_0010;
                    }

                }
                0xe8 => { // INX - Increment X Register
                    println!("register_x: {}", self.register_x);

                    if self.register_x < 0xff {
                        self.register_x = self.register_x + 1;
                    } else {
                        self.register_x = 0x00;
                    }

                    println!("register_x + 1: {}", self.register_x);

                    self.update_zero_and_negative_flags(self.register_x);
                }
                0x00 => return, // BRK
                _ => todo!()
            }
        }
    }

    fn lda(&mut self, value: u8) {
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn tax(&mut self) {
        println!("!!! tax: {} to {}", self.register_a, self.register_x);
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            // set 0 flag
            self.status = self.status | 0b0000_0010;
        } else {
            // unset of flag
            self.status = self.status & 0b1111_1101;
        }

        // true if if register_a has a 1 at bit 7 (most significant bit)
        if result & 0b1000_0000 != 0 {
            // updates the negative flag
            self.status = self.status | 0b1000_0000;
        } else {
            self.status = self.status & 0b0111_1111;
        }
    }

    fn set_carry_flag(&mut self) {
        self.status = self.status | 0b0000_0001;
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        // creating value from two bytes that are stored in Little-Endian 0x8000 -> 00 80
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16) // or the first 8 bytes of hi with low  
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8; // right shift -- drops lower 8 bits

        // bitwise AND against 11111111, mask all but lowest 8 bits,
        // extracting low bytes 00000000_1111111 & 10100000_10110001 -> 00000000_10110001 
        let low = (data & 0xff) as u8;

        self.mem_write(pos, low);
        self.mem_write(pos + 1, hi);
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);

        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status * 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();

        // load will call reset, do this before manipulating registers
        cpu.load(vec![0xaa, 0x00]);
        cpu.reset();
        cpu.register_a = 10;
        
        cpu.run();

        assert_eq!(cpu.register_x, 10)
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        // lda -> 0xc0
        // assign register a to register_x (TAX)
        // Increment X - INX (0xe8)
        // break
        cpu.interpret(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.register_x = 0xff;
        cpu.interpret(vec![0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }

    #[test]
    fn test_inx_sets_zero_flag() {
        let mut cpu = CPU::new();
        cpu.register_x = 0xff;
        // inc 1 will overflow to zero
        cpu.interpret(vec![0xe8, 0x00]);

        assert_eq!(cpu.status, 0b0000_0010);
    }
}

