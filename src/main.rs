use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::stdin;
use std::io::Read;
use std::io::Write;
use std::ops::BitAnd;
use std::ops::BitOr;
use std::ops::BitXor;
use std::ops::Div;
use std::ops::Not;
use std::ops::Rem;
use std::process::Stdio;
use std::str::SplitWhitespace;

type Value = u64;

#[derive(Clone, Copy, Debug)]
struct RegId(u8);

#[derive(Clone, Copy, Debug)]
struct Imm(Value);

#[derive(Clone, Copy, Debug)]
struct GlobalAddr(u32);

#[derive(Clone, Copy, Debug)]
struct FunctionAddr(u16);

#[derive(Clone, Copy, Debug)]
struct LocalAddr(u8);

#[derive(Debug)]
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

#[derive(Debug)]
enum Instruction {
    Output(RegId),
    LoadImm(RegId, Imm),
    LoadMem(RegId, GlobalAddr),
    StoreMem(RegId, GlobalAddr),
    LoadReg(RegId, RegId),
    StoreReg(RegId, RegId),
    Jmp(LocalAddr),
    Jo(RegId, LocalAddr),
    Call(FunctionAddr),
    CallReg(RegId),
    Return(),
    Swap(RegId, RegId),
    Op(Operation, RegId, RegId),
    OpImm(Operation, RegId, Imm),
}

fn assemble(text: String) -> Vec<u8> {
    todo!()
    // let mut data: Vec<u8> = Vec::new();

    // let mut labels: HashMap<String, usize> = HashMap::new();
    // let mut label_uses: Vec<(String, usize)> = Vec::new();

    // let encode_register = |words: &mut SplitWhitespace| -> u8 {
    //     let w = words.next().unwrap();
    //     assert!(w.starts_with("r"));
    //     let i = (&w[1..]).parse::<u8>().unwrap();
    //     i
    // };

    // let encode_address =
    //     |words: &mut SplitWhitespace, data: &mut Vec<u8>, label_uses: &mut Vec<(String, usize)>| {
    //         let w = words.next().unwrap();
    //         let [b0, b1] = if let Ok(i) = w.parse::<i16>() {
    //             i.to_be_bytes()
    //         } else {
    //             label_uses.push((w.to_string(), data.len()));
    //             [0, 0]
    //         };
    //         data.push(b0);
    //         data.push(b1);
    //     };

    // let encode_operation = |opstr: &str| -> u8 {
    //     match opstr {
    //         "copy" => 0b00000,
    //         "not" => 0b00001,
    //         "neg" => 0b00010,
    //         "reverse" => 0b00011,
    //         "numones" => 0b00100,
    //         "numzeros" => 0b00101,
    //         "and" => 0b00110,
    //         "or" => 0b00111,
    //         "xor" => 0b01000,
    //         "shl" => 0b01001,
    //         "shlm" => 0b01010,
    //         "shr" => 0b01011,
    //         "shrm" => 0b01100,
    //         "rotl" => 0b01101,
    //         "rotr" => 0b01110,
    //         "addc" => 0b01111,
    //         "addm" => 0b10000,
    //         "subc" => 0b10001,
    //         "subm" => 0b10010,
    //         "absdiff" => 0b10011,
    //         "mulc" => 0b10100,
    //         "mulm" => 0b10101,
    //         "div" => 0b10110,
    //         "mod" => 0b10111,
    //         "powm" => 0b11000,
    //         "powc" => 0b11001,
    //         "gt" => 0b11010,
    //         "ge" => 0b11011,
    //         "lt" => 0b11100,
    //         "le" => 0b11101,
    //         "eq" => 0b11110,
    //         "ne" => 0b11111,
    //         _ => panic!("{}", opstr),
    //     }
    // };

    // for line in text.lines() {
    //     let line = line.trim().to_string();
    //     let line = line.split(";").next().unwrap();
    //     if line.is_empty() {
    //         continue;
    //     }
    //     let mut words = line.split_whitespace();

    //     let first_word = words.next().unwrap();

    //     if first_word.ends_with(":") {
    //         let label_name = first_word[..(first_word.len() - 1)].to_string();
    //         labels.insert(label_name, data.len());
    //         continue;
    //     }

    //     match first_word {
    //         "output" => data.push(0b0000_0000 | encode_register(&mut words)),
    //         "outputw" => data.push(0b0001_0000 | encode_register(&mut words)),
    //         "loadmem" => {
    //             data.push(0b0010_0000 | encode_register(&mut words));
    //             encode_address(&mut words, &mut data, &mut label_uses);
    //         }
    //         "loadmemw" => {
    //             data.push(0b0011_0000 | encode_register(&mut words));
    //             encode_address(&mut words, &mut data, &mut label_uses);
    //         }
    //         "storemem" => {
    //             data.push(0b0100_0000 | encode_register(&mut words));
    //             encode_address(&mut words, &mut data, &mut label_uses);
    //         }
    //         "storememw" => {
    //             data.push(0b0101_0000 | encode_register(&mut words));
    //             encode_address(&mut words, &mut data, &mut label_uses);
    //         }
    //         "jmp" => {
    //             data.push(0b0110_0000);
    //             encode_address(&mut words, &mut data, &mut label_uses);
    //         }
    //         "jo" => {
    //             data.push(0b0111_0000 | encode_register(&mut words));
    //             encode_address(&mut words, &mut data, &mut label_uses);
    //         }
    //         _ => {
    //             let mut opstr = first_word.to_string();
    //             let mut wide = false;
    //             let mut immediate = false;
    //             if opstr.ends_with("w") {
    //                 opstr.remove(opstr.len() - 1);
    //                 wide = true;
    //             }
    //             if opstr.ends_with("imm") {
    //                 opstr.drain((opstr.len() - 3)..);
    //                 immediate = true;
    //             }
    //             let mut opcode = 0b1000_0000;
    //             if wide {
    //                 opcode |= 0b0010_0000;
    //             }
    //             if immediate {
    //                 opcode |= 0b0100_0000;
    //             }
    //             opcode |= encode_operation(&opstr);
    //             data.push(opcode);
    //             let a = encode_register(&mut words);
    //             let b = encode_register(&mut words);
    //             data.push((a << 4) | b);
    //             if immediate {
    //                 if wide {
    //                     let i = words.next().unwrap().parse::<WideValue>().unwrap();
    //                     for b in i.to_be_bytes() {
    //                         data.push(b);
    //                     }
    //                 } else {
    //                     let i = words.next().unwrap().parse::<Value>().unwrap();
    //                     for b in i.to_be_bytes() {
    //                         data.push(b);
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }

    // for (name, location) in label_uses {
    //     let value = *labels.get(&name).unwrap();
    //     let [m0, m1] = (value as u16).to_be_bytes();
    //     data[location + 0] = m0;
    //     data[location + 1] = m1;
    // }

    // data
}

struct Machine {
    memory: Vec<u8>,
    program_counter: usize,
    stack_pointer: usize,
    register_file: [Value; 16],
}

impl Machine {
    fn new(memory: Vec<u8>) -> Machine {
        assert!(!memory.is_empty());
        let stack_pointer = memory.len() - 1;
        Machine {
            memory,
            program_counter: 0,
            stack_pointer,
            register_file: [0; 16],
        }
    }

    fn run<T: Write>(&mut self, output: &mut T) {
        let num_bins: usize = 80;
        let interval: usize = 50000;
        let mut histogram = Vec::<usize>::new();
        histogram.resize(num_bins, 0);
        let mut interval_counter = 0;
        let chars: Vec<char> = " ._-+=!$#".chars().collect();
        loop {
            // let f0 = self.current_function_start();
            // let pc = self.program_counter;
            let mut i = self.fetch();
            // println!("{} {:?}", pc, i);
            // let f1 = self.current_function_start();
            // if f0 != f1 {
            //     i = Instruction::Return();
            // }
            self.execute(i, output);
            histogram[self.program_counter * num_bins / self.memory.len()] += 1;
            let max = histogram.iter().cloned().max().unwrap().max(1);
            if interval_counter == interval {
                interval_counter = 0;
                print!("|");
                for c in histogram.iter().cloned() {
                    let nc = c * (chars.len() - 1) / max;
                    if c > 0 && nc == 0 {
                        print!(".");
                    } else {
                        print!("{}", chars[nc]);
                    }
                }
                println!("|");
                for c in &mut histogram {
                    *c = 0;
                }
            }
            interval_counter += 1;
        }
    }

    fn fetch(&mut self) -> Instruction {
        let b0 = self.next_instruction_byte();
        let (n0a, n0b) = Self::byte_to_nibbles(b0);
        match n0a {
            0b0000 => Instruction::Output(RegId(n0b)),
            0b0001 => Instruction::LoadImm(RegId(n0b), self.fetch_imm()),
            0b0010 => Instruction::LoadMem(RegId(n0b), self.fetch_global_addr()),
            0b0011 => Instruction::StoreMem(RegId(n0b), self.fetch_global_addr()),
            0b0100 => Instruction::LoadReg(
                RegId(n0b),
                RegId(Self::byte_to_nibbles(self.next_instruction_byte()).0),
            ),
            0b0101 => Instruction::StoreReg(
                RegId(n0b),
                RegId(Self::byte_to_nibbles(self.next_instruction_byte()).0),
            ),
            0b0110 => Instruction::Jmp(self.fetch_local_addr()),
            0b0111 => Instruction::Jo(RegId(n0b), self.fetch_local_addr()),
            0b1000 => Instruction::Call(self.fetch_function_addr()),
            0b1001 => Instruction::CallReg(RegId(n0b)),
            0b1010 => Instruction::Return(),
            0b1011 => Instruction::Swap(
                RegId(n0b),
                RegId(Self::byte_to_nibbles(self.next_instruction_byte()).0),
            ),
            0b1100..=0b1111 => {
                let op = Self::decode_operation(((n0a & 1) << 4) | n0b);
                let ab = self.next_instruction_byte();
                let (a, b) = Self::byte_to_nibbles(ab);
                match n0a >> 1 {
                    0b110 => Instruction::Op(op, RegId(a), RegId(b)),
                    0b111 => Instruction::OpImm(op, RegId(a), self.fetch_imm()),
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
            Instruction::LoadImm(a, v) => (), // self.write_register(a, v.0),
            Instruction::LoadMem(a, m) => (), // self.write_register(a, self.read_memory(m)),
            Instruction::StoreMem(a, m) => (), // self.write_memory(m, self.read_register(a)),
            Instruction::LoadReg(a, b) => {
                // self.write_register(a, self.read_memory(GlobalAddr(self.read_register(b))))
            }
            Instruction::StoreReg(a, b) => {
                // self.write_memory(GlobalAddr(self.read_register(b)), self.read_register(a))
            }
            Instruction::Jmp(m) => {
                // TODO: what if this overflows?
                // self.program_counter = self.current_function_start() + m.0 as usize;
            }
            Instruction::Jo(a, m) => {
                let va = self.read_register(a);
                if va & 1 == 1 {
                    // TODO: what if this overflows?
                    // self.program_counter = self.current_function_start() + m.0 as usize;
                }
            }
            Instruction::Call(f) => {
                // self.do_call(f);
            }
            Instruction::CallReg(a) => {
                // self.do_call(FunctionAddr(self.read_register(a) as u16));
            }
            Instruction::Return() => {
                // self.do_return();
            }
            Instruction::Swap(a, b) => {
                let tmp = self.read_register(a);
                self.write_register(a, self.read_register(b));
                self.write_register(b, tmp);
            }
            Instruction::Op(o, a, b) => self.write_register(
                a,
                Self::evaluate_operation(o, self.read_register(a), self.read_register(b)),
            ),
            Instruction::OpImm(o, a, i) => {
                self.write_register(a, Self::evaluate_operation(o, self.read_register(a), i.0))
            }
        }
    }

    fn do_call(&mut self, f: FunctionAddr) {
        let addr = (((f.0 as usize) << 8) % self.memory.len()) & (!0xff);
        assert!(addr % 256 == 0);
        if addr > self.current_function_start() {
            self.push_addr(self.program_counter as u32);
            self.program_counter = addr;
            let b = self.next_instruction_byte();
            for i in 0..(b & 0xf) {
                self.push_value(self.read_register(RegId(i)));
            }
        }
    }

    fn do_return(&mut self) {
        let b = self.memory[self.current_function_start()];
        for i in (0..(b & 0xf)).rev() {
            let v = self.pop_value();
            self.write_register(RegId(i), v);
        }
        self.program_counter = (self.pop_addr() as usize) % self.memory.len();
    }

    fn push_byte(&mut self, byte: u8) {
        if self.stack_pointer == 0 {
            self.stack_pointer = self.memory.len();
        }
        self.stack_pointer -= 1;
        self.memory[self.stack_pointer] = byte;
    }

    fn pop_byte(&mut self) -> u8 {
        let b = self.memory[self.stack_pointer];
        self.stack_pointer += 1;
        if self.stack_pointer == self.memory.len() {
            self.stack_pointer = 0;
        }
        b
    }

    fn push_value(&mut self, value: Value) {
        for b in value.to_be_bytes() {
            self.push_byte(b);
        }
    }

    fn pop_value(&mut self) -> Value {
        let mut bytes = Value::default().to_be_bytes();
        for b in bytes.iter_mut().rev() {
            *b = self.pop_byte();
        }
        Value::from_be_bytes(bytes)
    }

    fn push_addr(&mut self, addr: u32) {
        for b in addr.to_be_bytes() {
            self.push_byte(b);
        }
    }

    fn pop_addr(&mut self) -> u32 {
        let mut bytes = u32::default().to_be_bytes();
        for b in bytes.iter_mut().rev() {
            *b = self.pop_byte();
        }
        u32::from_be_bytes(bytes)
    }

    fn current_function_start(&self) -> usize {
        self.program_counter & (!0xFF)
    }

    fn read_register(&self, register: RegId) -> Value {
        // HACK
        self.register_file[register.0 as usize & 3]
    }

    fn write_register(&mut self, register: RegId, value: Value) {
        // HACK
        self.register_file[register.0 as usize & 3] = value;
    }

    fn read_memory(&self, address: GlobalAddr) -> Value {
        let mut bytes = Value::default().to_be_bytes();
        let l = self.memory.len();
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = self.memory[(address.0 as usize + i) % l];
        }
        Value::from_be_bytes(bytes)
    }

    fn write_memory(&mut self, address: GlobalAddr, value: Value) {
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

    fn fetch_imm(&mut self) -> Imm {
        let mut bytes = Value::default().to_be_bytes();
        for b in &mut bytes {
            *b = self.next_instruction_byte();
        }
        Imm(Value::from_be_bytes(bytes))
    }

    fn fetch_global_addr(&mut self) -> GlobalAddr {
        GlobalAddr(u32::from_be_bytes([
            0,
            self.next_instruction_byte(),
            self.next_instruction_byte(),
            self.next_instruction_byte(),
        ]))
    }

    fn fetch_local_addr(&mut self) -> LocalAddr {
        LocalAddr(self.next_instruction_byte())
    }

    fn fetch_function_addr(&mut self) -> FunctionAddr {
        FunctionAddr(u16::from_be_bytes([
            self.next_instruction_byte(),
            self.next_instruction_byte(),
        ]))
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
}

fn main() {
    let args: Vec<_> = env::args().collect();

    // HACK for debugging
    // let args: Vec<String> = ["", "./example2.asm", "--assemble"]
    //     .iter()
    //     .map(|s| s.to_string())
    //     .collect();

    if args.len() < 2 || args.len() > 3 {
        println!("Usage: {} path/to/file.bin [--assemble]", args[0]);
        return;
    }
    let mut memory = if args[1] == "-" {
        let mut v = Vec::new();
        stdin().read_to_end(&mut v).unwrap();
        v
    } else {
        fs::read(&args[1]).unwrap()
    };
    if args.len() == 3 {
        if args[2] == "--assemble" {
            memory = assemble(String::from_utf8(memory).unwrap());
        } else {
            println!("What??");
            return;
        }
    }

    let mut aplay_process = std::process::Command::new("aplay")
        // .args(["-r", "44100", "-f", "S16_BE"])
        .args(["-c4", "-r16"])
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();

    let mut aplay_stdin = aplay_process.stdin.take().unwrap();

    let mut machine = Machine::new(memory);
    machine.run(&mut aplay_stdin);
}
