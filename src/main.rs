use std::env;
use std::fs;
use std::io::stdin;
use std::io::stdout;
use std::io::Read;
use std::io::Write;
use std::ops::BitAnd;
use std::ops::BitOr;
use std::ops::BitXor;
use std::ops::Div;
use std::ops::Not;
use std::ops::Rem;

#[derive(Clone, Copy)]
struct Reg8Id(u8);

#[derive(Clone, Copy)]
struct Reg16Id(u8);

#[derive(Clone, Copy)]
struct Imm8(u8);

#[derive(Clone, Copy)]
struct Imm16(u16);

#[derive(Clone, Copy)]
struct Addr(u16);

enum Operation {
    Copy,
    Not,
    Neg,
    Reverse,
    Numzeros,
    Numones,
    And,
    Or,
    Xor,
    Shl,
    Shlm,
    Shr,
    Shrm,
    Rotl,
    Rotr,
    Addc,
    Addm,
    Subc,
    Subm,
    Absdiff,
    Mulc,
    Mulm,
    Div,
    Mod,
    Powm,
    Powc,
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
    Ne,
}

enum Instruction {
    Output(Reg8Id),
    OutputW(Reg16Id),
    LoadMem(Reg8Id, Addr),
    LoadMemW(Reg16Id, Addr),
    StoreMem(Reg8Id, Addr),
    StoreMemW(Reg16Id, Addr),
    Jmp(Addr),
    Jo(Reg8Id, Addr),
    Op(Operation, Reg8Id, Reg8Id),
    OpW(Operation, Reg16Id, Reg16Id),
    OpImm(Operation, Reg8Id, Reg8Id, Imm8),
    OpImmW(Operation, Reg16Id, Reg16Id, Imm16),
}

struct Machine {
    memory: Vec<u8>,
    program_counter: usize,
    register_file: [u8; 16],
}

impl Machine {
    fn new(memory: Vec<u8>) -> Machine {
        assert!(!memory.is_empty());
        Machine {
            memory,
            program_counter: 0,
            register_file: [0; 16],
        }
    }

    fn run(&mut self) {
        loop {
            let i = self.fetch();
            self.execute(i);
        }
    }

    fn fetch(&mut self) -> Instruction {
        let b0 = self.next_instruction_byte();
        let (n0a, n0b) = Self::byte_to_nibbles(b0);
        match n0a {
            0b0000 => Instruction::Output(Reg8Id(n0b)),
            0b0001 => Instruction::OutputW(Reg16Id(n0b)),
            0b0010 => Instruction::LoadMem(
                Reg8Id(n0b),
                Self::bytes_to_addr(self.next_instruction_byte(), self.next_instruction_byte()),
            ),
            0b0011 => Instruction::LoadMemW(
                Reg16Id(n0b),
                Self::bytes_to_addr(self.next_instruction_byte(), self.next_instruction_byte()),
            ),
            0b0100 => Instruction::StoreMem(
                Reg8Id(n0b),
                Self::bytes_to_addr(self.next_instruction_byte(), self.next_instruction_byte()),
            ),
            0b0101 => Instruction::StoreMemW(
                Reg16Id(n0b),
                Self::bytes_to_addr(self.next_instruction_byte(), self.next_instruction_byte()),
            ),
            0b0110 => Instruction::Jmp(Addr(u16::from_be_bytes([
                self.next_instruction_byte(),
                self.next_instruction_byte(),
            ]))),
            0b0111 => Instruction::Jo(
                Reg8Id(n0b),
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
                    0b100 => Instruction::Op(op, Reg8Id(a), Reg8Id(b)),
                    0b101 => Instruction::OpW(op, Reg16Id(a), Reg16Id(b)),
                    0b110 => Instruction::OpImm(
                        op,
                        Reg8Id(a),
                        Reg8Id(b),
                        Imm8(self.next_instruction_byte()),
                    ),
                    0b111 => Instruction::OpImmW(
                        op,
                        Reg16Id(a),
                        Reg16Id(b),
                        Imm16(u16::from_be_bytes([
                            self.next_instruction_byte(),
                            self.next_instruction_byte(),
                        ])),
                    ),
                    _ => panic!(),
                }
            }
            _ => panic!(),
        }
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Output(a) => {
                let b = self.read_register_8bit(a);
                stdout().write(&[b]).unwrap();
            }
            Instruction::OutputW(a) => {
                let [b0, b1] = self.read_register_16bit(a).to_be_bytes();
                stdout().write(&[b0, b1]).unwrap();
            }
            Instruction::LoadMem(a, m) => self.write_register_8bit(a, self.read_memory_8bit(m)),
            Instruction::LoadMemW(a, m) => self.write_register_16bit(a, self.read_memory_16bit(m)),
            Instruction::StoreMem(a, m) => self.write_memory_8bit(m, self.read_register_8bit(a)),
            Instruction::StoreMemW(a, m) => self.write_memory_16bit(m, self.read_register_16bit(a)),
            Instruction::Jmp(m) => {
                self.program_counter = (m.0 as usize) % self.memory.len();
            }
            Instruction::Jo(a, m) => {
                if self.read_register_8bit(a) & 1 == 1 {
                    self.program_counter = (m.0 as usize) % self.memory.len();
                }
            }
            Instruction::Op(o, a, b) => self.write_register_8bit(
                a,
                Self::evaluate_operation_8bit(
                    o,
                    self.read_register_8bit(a),
                    self.read_register_8bit(b),
                ),
            ),
            Instruction::OpW(o, a, b) => self.write_register_16bit(
                a,
                Self::evaluate_operation_16bit(
                    o,
                    self.read_register_16bit(a),
                    self.read_register_16bit(b),
                ),
            ),
            Instruction::OpImm(o, a, b, i) => self.write_register_8bit(
                a,
                Self::evaluate_operation_8bit(o, self.read_register_8bit(b), i.0),
            ),
            Instruction::OpImmW(o, a, b, i) => self.write_register_16bit(
                a,
                Self::evaluate_operation_16bit(o, self.read_register_16bit(b), i.0),
            ),
        }
    }

    fn read_register_8bit(&self, register: Reg8Id) -> u8 {
        self.register_file[register.0 as usize]
    }
    fn read_register_16bit(&self, register: Reg16Id) -> u16 {
        u16::from_be_bytes([
            self.register_file[register.0 as usize],
            self.register_file[((register.0 as usize) + 1) & 0xf],
        ])
    }

    fn write_register_8bit(&mut self, register: Reg8Id, value: u8) {
        self.register_file[register.0 as usize] = value
    }
    fn write_register_16bit(&mut self, register: Reg16Id, value: u16) {
        let [b0, b1] = value.to_be_bytes();
        self.register_file[register.0 as usize] = b0;
        self.register_file[((register.0 as usize) + 1) & 0xf] = b1;
    }

    fn read_memory_8bit(&self, address: Addr) -> u8 {
        self.memory[(address.0 as usize) % self.memory.len()]
    }
    fn read_memory_16bit(&self, address: Addr) -> u16 {
        u16::from_be_bytes([
            self.memory[(address.0 as usize) % self.memory.len()],
            self.memory[(address.0 as usize + 1) % self.memory.len()],
        ])
    }

    fn write_memory_8bit(&mut self, address: Addr, value: u8) {
        let l = self.memory.len();
        self.memory[(address.0 as usize) % l] = value;
    }
    fn write_memory_16bit(&mut self, address: Addr, value: u16) {
        let [b0, b1] = value.to_be_bytes();
        let l = self.memory.len();
        self.memory[(address.0 as usize) % l] = b0;
        self.memory[(address.0 as usize + 1) % l] = b1;
    }

    fn next_instruction_byte(&mut self) -> u8 {
        let b = self.memory[self.program_counter];
        self.program_counter += 1;
        if self.program_counter >= self.memory.len() {
            self.program_counter = 0;
        }
        b
    }

    fn byte_to_nibbles(b: u8) -> (u8, u8) {
        (b & 0xf, b >> 4)
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

    fn evaluate_operation_8bit(op: Operation, a: u8, b: u8) -> u8 {
        match op {
            Operation::Copy => b,
            Operation::Not => b.not(),
            Operation::Neg => u8::MAX - b,
            Operation::Reverse => b.reverse_bits(),
            Operation::Numzeros => b.count_zeros() as u8,
            Operation::Numones => b.count_ones() as u8,
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
            Operation::Gt => a.gt(&b) as u8,
            Operation::Ge => a.ge(&b) as u8,
            Operation::Lt => a.lt(&b) as u8,
            Operation::Le => a.le(&b) as u8,
            Operation::Eq => a.eq(&b) as u8,
            Operation::Ne => a.ne(&b) as u8,
        }
    }

    fn evaluate_operation_16bit(op: Operation, a: u16, b: u16) -> u16 {
        match op {
            Operation::Copy => b,
            Operation::Not => b.not(),
            Operation::Neg => u16::MAX - b,
            Operation::Reverse => b.reverse_bits(),
            Operation::Numzeros => b.count_zeros() as u16,
            Operation::Numones => b.count_ones() as u16,
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
            Operation::Gt => a.gt(&b) as u16,
            Operation::Ge => a.ge(&b) as u16,
            Operation::Lt => a.lt(&b) as u16,
            Operation::Le => a.le(&b) as u16,
            Operation::Eq => a.eq(&b) as u16,
            Operation::Ne => a.ne(&b) as u16,
        }
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} path/to/file.bin", args[0]);
        return;
    }
    let memory = if args[1] == "-" {
        let mut v = Vec::new();
        stdin().read_to_end(&mut v).unwrap();
        v
    } else {
        fs::read(&args[1]).unwrap()
    };
    let mut machine = Machine::new(memory);
    machine.run();
}
