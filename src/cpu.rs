pub struct CPU {
   pub register_a: u8,
   pub status: u8,
   pub program_counter: u16,
   pub register_x: u8,
}

impl CPU {
   pub fn new() -> Self {
      CPU {
         register_a: 0,
         register_x: 0,
         status: 0,
         program_counter: 0,
      }
   }

   pub fn interpret(&mut self, program: Vec<u8>) {
      self.program_counter = 0;

      loop {
         let opscode = program[self.program_counter as usize];
         self.program_counter += 1;

         match opscode {
            0xA9 => { // LDA
               let param = program[self.program_counter as usize];
               self.program_counter += 1;
               self.register_a = param;

               if self.register_a == 0 {
                  // set 0 flag
                  self.status = self.status | 0b0000_0010;
               } else {
                  // unset of flag
                  self.status = self.status & 0b1111_1101;
               }

               // true if if register_a has a 1 at bit 7 (most significant bit)
               if self.register_a & 0b1000_0000 != 0 {
                  // updates the negative flag
                  self.status = self.status | 0b1000_0000;
               } else {
                  self.status = self.status & 0b0111_1111;
               }
            }
            0x00 => { // BRK
               return;
            }
            0xAA => { // TAX
               // copy a to x
               self.register_x = self.register_a;

               // update the status register 
               if self.register_x == 0 {
                  // set the 0 flag
                  self.status = self.status | 0b0000_0010;
               } else {
                  // toggle off the 0 flag, leave other set flags alone
                  self.status = self.status & 0b1111_1101;
               }

               // if bit 7 on register x is set, set negative flag
               if self.register_x & 0b1000_000 != 0 {
                  self.status = self.status | 0b1000_000;
               } else {
                  // toggle off the negative flag, leave other set flags alone
                  self.status = self.status & 0b0111_1111;
               }
            }
            _ => todo!()
         }
      }
   }
}


#[cfg(test)]
mod test {
   use super::*;

   #[test]
   fn test_0xa9_lda_immediate_load_data() {
      let mut cpu = CPU::new();
      cpu.interpret(vec![0xa9, 0x05, 0x00]);

      assert_eq!(cpu.register_a, 0x05);
      assert!(cpu.status & 0b0000_0010 == 0b00);
      assert!(cpu.status * 0b1000_0000 == 0);
   }

   #[test]
   fn test_0xa9_lda_zero_flag() {
      let mut cpu = CPU::new();
      cpu.interpret(vec![0xa9, 0x00, 0x00]);
      assert!(cpu.status & 0b0000_0010 == 0b10);
   }

   #[test]
   fn test_0xaa_tax_move_a_to_x() {
      let mut cpu = CPU::new();
      cpu.register_a = 10;
      cpu.interpret(vec![0xaa, 0x00]);

      assert_eq!(cpu.register_x, 10)
   }
}

