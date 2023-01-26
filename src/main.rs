use std::env;
use std::fs;
use std::io::stdin;
use std::io::stdout;
use std::io::Read;
use std::io::Write;
use std::ops::BitAnd;
use std::ops::BitOr;
use std::ops::BitXor;
use std::ops::Rem;
use std::ops::Shl;
use std::ops::Shr;

#[derive(Clone, Copy)]
struct RegId(u8);
struct Imm(u8);
struct Offset(u8);
struct Addr(u16);

enum Operation {
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Rotl,
    Rotr,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Gt,
    Lt,
    Eq,
    Neq,
}

enum Instruction {
    Output(RegId),
    LoadImm(RegId, Imm),
    LoadMem(RegId, Addr),
    StoreMem(RegId, Addr),
    LoadInd(RegId, RegId, RegId),
    StoreInd(RegId, RegId, RegId),
    Call(Addr),
    Return(),
    Jmp(Addr),
    JmpFwdO(RegId, Offset),
    JmpBwdO(RegId, Offset),
    JmpO(RegId, Addr),
    Push(RegId),
    Pop(RegId),
    OpImm(Operation, RegId, RegId, Imm),
    Op(Operation, RegId, RegId),
}

struct Machine {
    memory: Vec<u8>,
    program_counter: usize,
    stack_pointer: usize,
    register_file: [u8; 16],
}

impl Machine {
    fn new(memory: Vec<u8>) -> Machine {
        assert!(!memory.is_empty());
        Machine {
            memory,
            program_counter: 0,
            stack_pointer: 0, // TODO: ummmm
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
            0x0 => Instruction::Output(RegId(n0b)),
            0x1 => {
                let v = self.next_instruction_byte();
                Instruction::LoadImm(RegId(n0b), Imm(v))
            }
            0x2 => {
                let m0 = self.next_instruction_byte();
                let m1 = self.next_instruction_byte();
                Instruction::LoadMem(RegId(n0b), Self::bytes_to_addr(m0, m1))
            }
            0x3 => {
                let m0 = self.next_instruction_byte();
                let m1 = self.next_instruction_byte();
                Instruction::StoreMem(RegId(n0b), Self::bytes_to_addr(m0, m1))
            }
            0x4 => {
                let bc = self.next_instruction_byte();
                let (b, c) = Self::byte_to_nibbles(bc);
                Instruction::LoadInd(RegId(n0b), RegId(b), RegId(c))
            }
            0x5 => {
                let bc = self.next_instruction_byte();
                let (b, c) = Self::byte_to_nibbles(bc);
                Instruction::StoreInd(RegId(n0b), RegId(b), RegId(c))
            }
            0x6 => {
                let m0 = self.next_instruction_byte();
                let m1 = self.next_instruction_byte();
                Instruction::Call(Self::bytes_to_addr(m0, m1))
            }
            0x7 => Instruction::Return(),
            0x8 => {
                let m0 = self.next_instruction_byte();
                let m1 = self.next_instruction_byte();
                Instruction::Jmp(Self::bytes_to_addr(m0, m1))
            }
            0x9 => {
                let o = self.next_instruction_byte();
                Instruction::JmpFwdO(RegId(n0b), Offset(o))
            }
            0xa => {
                let o = self.next_instruction_byte();
                Instruction::JmpBwdO(RegId(n0b), Offset(o))
            }
            0xb => {
                let m0 = self.next_instruction_byte();
                let m1 = self.next_instruction_byte();
                Instruction::JmpO(RegId(n0b), Self::bytes_to_addr(m0, m1))
            }
            0xc => Instruction::Push(RegId(n0b)),
            0xd => Instruction::Pop(RegId(n0b)),
            0xe => {
                let ab = self.next_instruction_byte();
                let (a, b) = Self::byte_to_nibbles(ab);
                let v = self.next_instruction_byte();
                Instruction::OpImm(Self::nibble_to_operation(n0b), RegId(a), RegId(b), Imm(v))
            }
            0xf => {
                let ab = self.next_instruction_byte();
                let (a, b) = Self::byte_to_nibbles(ab);
                Instruction::Op(Self::nibble_to_operation(n0b), RegId(a), RegId(b))
            }
            _ => panic!(),
        }
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Output(a) => {
                stdout().write(&[self.read_register(a)]).unwrap();
            }
            Instruction::LoadImm(a, v) => self.write_register(a, v.0),
            Instruction::LoadMem(a, m) => self.write_register(a, self.read_memory(m)),
            Instruction::StoreMem(a, m) => self.write_memory(m, self.read_register(a)),
            Instruction::LoadInd(a, b, c) => {
                let m0 = self.read_register(b);
                let m1 = self.read_register(c);
                self.write_register(a, self.read_memory(Self::bytes_to_addr(m0, m1)))
            }
            Instruction::StoreInd(a, b, c) => {
                let m0 = self.read_register(b);
                let m1 = self.read_register(c);
                self.write_memory(Self::bytes_to_addr(m0, m1), self.read_register(a))
            }
            Instruction::Call(m) => {
                // let (a, b) = Self::address_to_bytes((self.program_counter & 0xffff) as u16);
                // self.push_stack(a);
                // self.push_stack(b);
                // self.program_counter = m.0 as usize;
            }
            Instruction::Return() => {
                // let b = self.pop_stack();
                // let a = self.pop_stack();
                // self.program_counter = Self::bytes_to_addr(a, b).0 as usize;
            }
            Instruction::Jmp(m) => {
                // self.program_counter = m.0 as usize;
            }
            Instruction::JmpFwdO(a, o) => {
                // if self.read_register(a) & 1 == 1 {
                //     self.program_counter += o.0 as usize;
                //     self.program_counter %= self.memory.len();
                // }
            }
            Instruction::JmpBwdO(a, o) => {
                // if self.read_register(a) & 1 == 1 {
                //     self.program_counter -= o.0 as usize;
                //     self.program_counter %= self.memory.len();
                // }
            }
            Instruction::JmpO(a, m) => {
                // if self.read_register(a) & 1 == 1 {
                //     self.program_counter = m.0 as usize;
                // }
            }
            Instruction::Push(a) => self.push_stack(self.read_register(a)),
            Instruction::Pop(a) => {
                let v = self.pop_stack();
                self.write_register(a, v)
            }
            Instruction::OpImm(op, a, b, v) => {
                let r = Self::evaluate_operation(op, self.read_register(b), v.0);
                self.write_register(a, r);
            }
            Instruction::Op(op, a, b) => {
                let r = Self::evaluate_operation(op, self.read_register(a), self.read_register(b));
                self.write_register(a, r);
            }
        }
    }

    fn read_register(&self, register: RegId) -> u8 {
        self.register_file[register.0 as usize]
    }

    fn write_register(&mut self, register: RegId, value: u8) {
        self.register_file[register.0 as usize] = value
    }

    fn read_memory(&self, address: Addr) -> u8 {
        self.memory[(address.0 as usize) % self.memory.len()]
    }

    fn write_memory(&mut self, address: Addr, value: u8) {
        let l = self.memory.len();
        self.memory[(address.0 as usize) % l] = value;
    }

    fn push_stack(&mut self, value: u8) {
        self.memory[self.stack_pointer as usize] = value;
        self.stack_pointer += 1;
        if self.stack_pointer >= self.memory.len() {
            self.stack_pointer = 0;
        }
    }

    fn pop_stack(&mut self) -> u8 {
        let v = self.memory[self.stack_pointer as usize];
        if self.stack_pointer == 0 {
            self.stack_pointer = self.memory.len();
        }
        self.stack_pointer -= 1;
        v
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

    fn address_to_bytes(address: u16) -> (u8, u8) {
        let [a, b] = address.to_le_bytes();
        (a, b)
    }

    fn nibble_to_operation(n: u8) -> Operation {
        match n {
            0x0 => Operation::And,
            0x1 => Operation::Or,
            0x2 => Operation::Xor,
            0x3 => Operation::Shl,
            0x4 => Operation::Shr,
            0x5 => Operation::Rotl,
            0x6 => Operation::Rotr,
            0x7 => Operation::Add,
            0x8 => Operation::Sub,
            0x9 => Operation::Mul,
            0xa => Operation::Div,
            0xb => Operation::Mod,
            0xc => Operation::Gt,
            0xd => Operation::Lt,
            0xe => Operation::Eq,
            0xf => Operation::Neq,
            _ => panic!(),
        }
    }

    fn evaluate_operation(op: Operation, a: u8, b: u8) -> u8 {
        match op {
            Operation::And => a.bitand(b),
            Operation::Or => a.bitor(b),
            Operation::Xor => a.bitxor(b),
            Operation::Shl => a.overflowing_shl(b as u32).0,
            Operation::Shr => a.overflowing_shr(b as u32).0,
            Operation::Rotl => a.rotate_left(b as u32),
            Operation::Rotr => a.rotate_right(b as u32),
            Operation::Add => a.overflowing_add(b).0,
            Operation::Sub => a.overflowing_sub(b).0,
            Operation::Mul => a.overflowing_mul(b).0,
            Operation::Div => a.overflowing_div(b.max(1)).0,
            Operation::Mod => a.rem(b.max(1)),
            Operation::Gt => (a > b) as u8,
            Operation::Lt => (a < b) as u8,
            Operation::Eq => (a == b) as u8,
            Operation::Neq => (a != b) as u8,
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
