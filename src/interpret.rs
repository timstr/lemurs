use std::{
    env, fs,
    io::{stdin, Read},
    process::Stdio,
};

use lemurs::{instruction::assemble, machine::Machine};

fn main() {
    let args: Vec<_> = env::args().collect();

    // HACK for debugging
    // let args: Vec<String> = ["", "./example2.asm", "--assemble"]
    //     .iter()
    //     .map(|s| s.to_string())
    //     .collect();

    if args.len() < 2 || args.len() > 3 {
        println!("Usage:");
        println!("  Run a binary file in the interpreter:");
        println!("   {} path/to/file.bin", args[0]);
        println!("");
        println!("  Assemble a program and run in the interpreter:");
        println!("   {} path/to/file.asm --assemble", args[0]);
        println!("");
        println!("  To receive a binary from stdin until EOF to run in the interpreter:");
        println!("   {} -", args[0]);
        println!("");
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
        .args(["-c2", "-r64"])
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();

    let mut aplay_stdin = aplay_process.stdin.take().unwrap();

    let mut machine = Machine::new(memory);
    loop {
        machine.run(2048, &mut aplay_stdin);
    }
}
