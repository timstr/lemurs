use std::{collections::HashMap, str::SplitWhitespace};

pub type Value = u32;
pub type WideValue = u64;

#[derive(Clone, Copy)]
pub struct RegId(pub u8);

#[derive(Clone, Copy)]
pub struct RegWId(pub u8);

#[derive(Clone, Copy)]
pub struct Imm(pub Value);

#[derive(Clone, Copy)]
pub struct ImmW(pub WideValue);

#[derive(Clone, Copy)]
pub struct Addr(pub u16);

pub enum Operation {
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

pub enum Instruction {
    Output(RegId),
    OutputW(RegWId),
    LoadMem(RegId, Addr),
    LoadMemW(RegWId, Addr),
    StoreMem(RegId, Addr),
    StoreMemW(RegWId, Addr),
    Jmp(Addr),
    Jo(RegId, Addr),
    Op(Operation, RegId, RegId),
    OpW(Operation, RegWId, RegWId),
    OpImm(Operation, RegId, RegId, Imm),
    OpImmW(Operation, RegWId, RegWId, ImmW),
}

pub fn assemble(text: String) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::new();

    let mut labels: HashMap<String, usize> = HashMap::new();
    let mut label_uses: Vec<(String, usize)> = Vec::new();

    let encode_register = |words: &mut SplitWhitespace| -> u8 {
        let w = words.next().unwrap();
        assert!(w.starts_with("r"));
        let i = (&w[1..]).parse::<u8>().unwrap();
        i
    };

    let encode_address =
        |words: &mut SplitWhitespace, data: &mut Vec<u8>, label_uses: &mut Vec<(String, usize)>| {
            let w = words.next().unwrap();
            let [b0, b1] = if let Ok(i) = w.parse::<i16>() {
                i.to_be_bytes()
            } else {
                label_uses.push((w.to_string(), data.len()));
                [0, 0]
            };
            data.push(b0);
            data.push(b1);
        };

    let encode_operation = |opstr: &str| -> u8 {
        match opstr {
            "copy" => 0b00000,
            "not" => 0b00001,
            "neg" => 0b00010,
            "reverse" => 0b00011,
            "numones" => 0b00100,
            "numzeros" => 0b00101,
            "and" => 0b00110,
            "or" => 0b00111,
            "xor" => 0b01000,
            "shl" => 0b01001,
            "shlm" => 0b01010,
            "shr" => 0b01011,
            "shrm" => 0b01100,
            "rotl" => 0b01101,
            "rotr" => 0b01110,
            "addc" => 0b01111,
            "addm" => 0b10000,
            "subc" => 0b10001,
            "subm" => 0b10010,
            "absdiff" => 0b10011,
            "mulc" => 0b10100,
            "mulm" => 0b10101,
            "div" => 0b10110,
            "mod" => 0b10111,
            "powm" => 0b11000,
            "powc" => 0b11001,
            "gt" => 0b11010,
            "ge" => 0b11011,
            "lt" => 0b11100,
            "le" => 0b11101,
            "eq" => 0b11110,
            "ne" => 0b11111,
            _ => panic!("{}", opstr),
        }
    };

    for line in text.lines() {
        let line = line.trim().to_string();
        let line = line.split(";").next().unwrap();
        if line.is_empty() {
            continue;
        }
        let mut words = line.split_whitespace();

        let first_word = words.next().unwrap();

        if first_word.ends_with(":") {
            let label_name = first_word[..(first_word.len() - 1)].to_string();
            labels.insert(label_name, data.len());
            continue;
        }

        match first_word {
            "output" => data.push(0b0000_0000 | encode_register(&mut words)),
            "outputw" => data.push(0b0001_0000 | encode_register(&mut words)),
            "loadmem" => {
                data.push(0b0010_0000 | encode_register(&mut words));
                encode_address(&mut words, &mut data, &mut label_uses);
            }
            "loadmemw" => {
                data.push(0b0011_0000 | encode_register(&mut words));
                encode_address(&mut words, &mut data, &mut label_uses);
            }
            "storemem" => {
                data.push(0b0100_0000 | encode_register(&mut words));
                encode_address(&mut words, &mut data, &mut label_uses);
            }
            "storememw" => {
                data.push(0b0101_0000 | encode_register(&mut words));
                encode_address(&mut words, &mut data, &mut label_uses);
            }
            "jmp" => {
                data.push(0b0110_0000);
                encode_address(&mut words, &mut data, &mut label_uses);
            }
            "jo" => {
                data.push(0b0111_0000 | encode_register(&mut words));
                encode_address(&mut words, &mut data, &mut label_uses);
            }
            _ => {
                let mut opstr = first_word.to_string();
                let mut wide = false;
                let mut immediate = false;
                if opstr.ends_with("w") {
                    opstr.remove(opstr.len() - 1);
                    wide = true;
                }
                if opstr.ends_with("imm") {
                    opstr.drain((opstr.len() - 3)..);
                    immediate = true;
                }
                let mut opcode = 0b1000_0000;
                if wide {
                    opcode |= 0b0010_0000;
                }
                if immediate {
                    opcode |= 0b0100_0000;
                }
                opcode |= encode_operation(&opstr);
                data.push(opcode);
                let a = encode_register(&mut words);
                let b = encode_register(&mut words);
                data.push((a << 4) | b);
                if immediate {
                    if wide {
                        let i = words.next().unwrap().parse::<WideValue>().unwrap();
                        for b in i.to_be_bytes() {
                            data.push(b);
                        }
                    } else {
                        let i = words.next().unwrap().parse::<Value>().unwrap();
                        for b in i.to_be_bytes() {
                            data.push(b);
                        }
                    }
                }
            }
        }
    }

    for (name, location) in label_uses {
        let value = *labels.get(&name).unwrap();
        let [m0, m1] = (value as u16).to_be_bytes();
        data[location + 0] = m0;
        data[location + 1] = m1;
    }

    data
}
