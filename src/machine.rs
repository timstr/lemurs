use std::{
    io::Write,
    ops::{BitAnd, BitOr, BitXor, Div, Not, Rem},
};

use crate::instruction::{
    Addr, Imm, ImmW, Instruction, Operation, RegId, RegWId, Value, WideValue,
};

pub struct Machine {
    memory: Vec<u8>,
    program_counter: usize,
    register_file: [u8; 256],
}

impl Machine {
    pub fn new(memory: Vec<u8>) -> Machine {
        assert!(!memory.is_empty());
        Machine {
            memory,
            program_counter: 0,
            register_file: [0; 256],
        }
    }

    pub fn run<T: Write>(&mut self, num_steps: usize, output: &mut T) {
        for _ in 0..num_steps {
            let i = self.fetch();
            self.execute(i, output);
        }
    }

    fn fetch(&mut self) -> Instruction {
        let b0 = self.next_instruction_byte();
        let (n0a, n0b) = Self::byte_to_nibbles(b0);
        match n0a {
            0b0000 => Instruction::Output(RegId(n0b)),
            0b0001 => Instruction::OutputW(RegWId(n0b)),
            0b0010 => Instruction::LoadMem(
                RegId(n0b),
                Self::bytes_to_addr(self.next_instruction_byte(), self.next_instruction_byte()),
            ),
            0b0011 => Instruction::LoadMemW(
                RegWId(n0b),
                Self::bytes_to_addr(self.next_instruction_byte(), self.next_instruction_byte()),
            ),
            0b0100 => Instruction::StoreMem(
                RegId(n0b),
                Self::bytes_to_addr(self.next_instruction_byte(), self.next_instruction_byte()),
            ),
            0b0101 => Instruction::StoreMemW(
                RegWId(n0b),
                Self::bytes_to_addr(self.next_instruction_byte(), self.next_instruction_byte()),
            ),
            0b0110 => Instruction::Jmp(Addr(u16::from_be_bytes([
                self.next_instruction_byte(),
                self.next_instruction_byte(),
            ]))),
            0b0111 => Instruction::Jo(
                RegId(n0b),
                Addr(u16::from_be_bytes([
                    self.next_instruction_byte(),
                    self.next_instruction_byte(),
                ])),
            ),
            0b1000..=0b1111 => {
                let op = Self::decode_operation(((n0a & 1) << 4) | n0b);
                let ab = self.next_instruction_byte();
                let (a, b) = Self::byte_to_nibbles(ab);
                match n0a >> 1 {
                    0b100 => Instruction::Op(op, RegId(a), RegId(b)),
                    0b101 => Instruction::OpW(op, RegWId(a), RegWId(b)),
                    0b110 => {
                        let mut bytes = Value::default().to_be_bytes();
                        for b in &mut bytes {
                            *b = self.next_instruction_byte();
                        }
                        Instruction::OpImm(op, RegId(a), RegId(b), Imm(Value::from_be_bytes(bytes)))
                    }
                    0b111 => {
                        let mut bytes = WideValue::default().to_be_bytes();
                        for b in &mut bytes {
                            *b = self.next_instruction_byte();
                        }
                        Instruction::OpImmW(
                            op,
                            RegWId(a),
                            RegWId(b),
                            ImmW(WideValue::from_be_bytes(bytes)),
                        )
                    }
                    _ => panic!(),
                }
            }
            _ => panic!(),
        }
    }

    fn execute<T: Write>(&mut self, instruction: Instruction, output: &mut T) {
        match instruction {
            Instruction::Output(a) => {
                let b = self.read_register(a);
                output.write(&[(b & 0xff) as u8]).unwrap();
            }
            Instruction::OutputW(a) => {
                let [b0, b1] = ((self.read_register_wide(a) & 0xffff) as u16).to_be_bytes();
                output.write(&[b0, b1]).unwrap();
            }
            Instruction::LoadMem(a, m) => self.write_register(a, self.read_memory(m)),
            Instruction::LoadMemW(a, m) => self.write_register_wide(a, self.read_memory_wide(m)),
            Instruction::StoreMem(a, m) => self.write_memory(m, self.read_register(a)),
            Instruction::StoreMemW(a, m) => self.write_memory_wide(m, self.read_register_wide(a)),
            Instruction::Jmp(m) => {
                self.program_counter = (m.0 as usize) % self.memory.len();
            }
            Instruction::Jo(a, m) => {
                let va = self.read_register(a);
                if va & 1 == 1 {
                    self.program_counter = (m.0 as usize) % self.memory.len();
                }
            }
            Instruction::Op(o, a, b) => self.write_register(
                a,
                Self::evaluate_operation(o, self.read_register(a), self.read_register(b)),
            ),
            Instruction::OpW(o, a, b) => self.write_register_wide(
                a,
                Self::evaluate_operation_wide(
                    o,
                    self.read_register_wide(a),
                    self.read_register_wide(b),
                ),
            ),
            Instruction::OpImm(o, a, b, i) => {
                self.write_register(a, Self::evaluate_operation(o, self.read_register(b), i.0))
            }
            Instruction::OpImmW(o, a, b, i) => self.write_register_wide(
                a,
                Self::evaluate_operation_wide(o, self.read_register_wide(b), i.0),
            ),
        }
    }

    fn read_register(&self, register: RegId) -> Value {
        let mut bytes = Value::default().to_be_bytes();
        let num_bytes = bytes.len();
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = self.register_file[(register.0 as usize) * num_bytes + i];
        }
        Value::from_be_bytes(bytes)
    }
    fn read_register_wide(&self, register: RegWId) -> WideValue {
        let mut bytes = WideValue::default().to_be_bytes();
        let num_bytes = bytes.len();
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = self.register_file[(register.0 as usize) * num_bytes + i];
        }
        WideValue::from_be_bytes(bytes)
    }

    fn write_register(&mut self, register: RegId, value: Value) {
        let bytes = value.to_be_bytes();
        let num_bytes = bytes.len();
        for (i, b) in bytes.into_iter().enumerate() {
            self.register_file[(register.0 as usize) * num_bytes + i] = b;
        }
    }
    fn write_register_wide(&mut self, register: RegWId, value: WideValue) {
        let bytes = value.to_be_bytes();
        let num_bytes = bytes.len();
        for (i, b) in bytes.into_iter().enumerate() {
            self.register_file[(register.0 as usize) * num_bytes + i] = b;
        }
    }

    fn read_memory(&self, address: Addr) -> Value {
        let mut bytes = Value::default().to_be_bytes();
        let l = self.memory.len();
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = self.memory[(address.0 as usize + i) % l];
        }
        Value::from_be_bytes(bytes)
    }
    fn read_memory_wide(&self, address: Addr) -> WideValue {
        let mut bytes = WideValue::default().to_be_bytes();
        let l = self.memory.len();
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = self.memory[(address.0 as usize + i) % l];
        }
        WideValue::from_be_bytes(bytes)
    }

    fn write_memory(&mut self, address: Addr, value: Value) {
        let l = self.memory.len();
        for (i, b) in value.to_be_bytes().into_iter().enumerate() {
            self.memory[(address.0 as usize + i) % l] = b;
        }
    }
    fn write_memory_wide(&mut self, address: Addr, value: WideValue) {
        let l = self.memory.len();
        for (i, b) in value.to_be_bytes().into_iter().enumerate() {
            self.memory[(address.0 as usize + i) % l] = b;
        }
    }

    fn next_instruction_byte(&mut self) -> u8 {
        let b = self.memory[self.program_counter];
        self.program_counter += 1;
        if self.program_counter >= self.memory.len() {
            // panic!();
            self.program_counter = 0;
        }
        b
    }

    fn byte_to_nibbles(b: u8) -> (u8, u8) {
        ((b >> 4) & 0xf, b & 0xf)
    }

    fn bytes_to_addr(b0: u8, b1: u8) -> Addr {
        Addr(u16::from_be_bytes([b0, b1]))
    }

    fn decode_operation(n: u8) -> Operation {
        match n {
            0b00000 => Operation::Copy,
            0b00001 => Operation::Not,
            0b00010 => Operation::Neg,
            0b00011 => Operation::Reverse,
            0b00100 => Operation::Numzeros,
            0b00101 => Operation::Numones,
            0b00110 => Operation::And,
            0b00111 => Operation::Or,
            0b01000 => Operation::Xor,
            0b01001 => Operation::Shl,
            0b01010 => Operation::Shlm,
            0b01011 => Operation::Shr,
            0b01100 => Operation::Shrm,
            0b01101 => Operation::Rotl,
            0b01110 => Operation::Rotr,
            0b01111 => Operation::Addc,
            0b10000 => Operation::Addm,
            0b10001 => Operation::Subc,
            0b10010 => Operation::Subm,
            0b10011 => Operation::Absdiff,
            0b10100 => Operation::Mulc,
            0b10101 => Operation::Mulm,
            0b10110 => Operation::Div,
            0b10111 => Operation::Mod,
            0b11000 => Operation::Powm,
            0b11001 => Operation::Powc,
            0b11010 => Operation::Gt,
            0b11011 => Operation::Ge,
            0b11100 => Operation::Lt,
            0b11101 => Operation::Le,
            0b11110 => Operation::Eq,
            0b11111 => Operation::Ne,
            _ => panic!(),
        }
    }

    fn evaluate_operation(op: Operation, a: Value, b: Value) -> Value {
        match op {
            Operation::Copy => b,
            Operation::Not => b.not(),
            Operation::Neg => Value::MAX - b,
            Operation::Reverse => b.reverse_bits(),
            Operation::Numzeros => b.count_zeros() as Value,
            Operation::Numones => b.count_ones() as Value,
            Operation::And => a.bitand(b),
            Operation::Or => a.bitor(b),
            Operation::Xor => a.bitxor(b),
            Operation::Shl => a.checked_shl(b as u32).unwrap_or(0),
            Operation::Shlm => a.wrapping_shl(b as u32),
            Operation::Shr => a.checked_shr(b as u32).unwrap_or(0),
            Operation::Shrm => a.wrapping_shr(b as u32),
            Operation::Rotl => a.rotate_left(b as u32),
            Operation::Rotr => a.rotate_right(b as u32),
            Operation::Addc => a.saturating_add(b),
            Operation::Addm => a.wrapping_add(b),
            Operation::Subc => a.saturating_sub(b),
            Operation::Subm => a.wrapping_sub(b),
            Operation::Absdiff => a.abs_diff(b),
            Operation::Mulc => a.saturating_mul(b),
            Operation::Mulm => a.wrapping_mul(b),
            Operation::Div => a.div(b.max(1)),
            Operation::Mod => a.rem(b.max(1)),
            Operation::Powm => a.saturating_pow(b as u32),
            Operation::Powc => a.wrapping_pow(b as u32),
            Operation::Gt => a.gt(&b) as Value,
            Operation::Ge => a.ge(&b) as Value,
            Operation::Lt => a.lt(&b) as Value,
            Operation::Le => a.le(&b) as Value,
            Operation::Eq => a.eq(&b) as Value,
            Operation::Ne => a.ne(&b) as Value,
        }
    }

    fn evaluate_operation_wide(op: Operation, a: WideValue, b: WideValue) -> WideValue {
        match op {
            Operation::Copy => b,
            Operation::Not => b.not(),
            Operation::Neg => WideValue::MAX - b,
            Operation::Reverse => b.reverse_bits(),
            Operation::Numzeros => b.count_zeros() as WideValue,
            Operation::Numones => b.count_ones() as WideValue,
            Operation::And => a.bitand(b),
            Operation::Or => a.bitor(b),
            Operation::Xor => a.bitxor(b),
            Operation::Shl => a.checked_shl(b as u32).unwrap_or(0),
            Operation::Shlm => a.wrapping_shl(b as u32),
            Operation::Shr => a.checked_shr(b as u32).unwrap_or(0),
            Operation::Shrm => a.wrapping_shr(b as u32),
            Operation::Rotl => a.rotate_left(b as u32),
            Operation::Rotr => a.rotate_right(b as u32),
            Operation::Addc => a.saturating_add(b),
            Operation::Addm => a.wrapping_add(b),
            Operation::Subc => a.saturating_sub(b),
            Operation::Subm => a.wrapping_sub(b),
            Operation::Absdiff => a.abs_diff(b),
            Operation::Mulc => a.saturating_mul(b),
            Operation::Mulm => a.wrapping_mul(b),
            Operation::Div => a.div(b.max(1)),
            Operation::Mod => a.rem(b.max(1)),
            Operation::Powm => a.saturating_pow(b as u32),
            Operation::Powc => a.wrapping_pow(b as u32),
            Operation::Gt => a.gt(&b) as WideValue,
            Operation::Ge => a.ge(&b) as WideValue,
            Operation::Lt => a.lt(&b) as WideValue,
            Operation::Le => a.le(&b) as WideValue,
            Operation::Eq => a.eq(&b) as WideValue,
            Operation::Ne => a.ne(&b) as WideValue,
        }
    }
}
