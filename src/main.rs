use std::{panic, process};

use lemurs::lemurs_app::LemursApp;

// TODO:
// - keep a set of strings of bytes representing the current population of programs
// - repeat:
//    - randomly mutate those programs to define a new generation
//    - evaluate all programs until they've produced a sufficient amount of output or timed out
//    - display the output of each program as a set of spectrograms
//    - click each spectrogram to listen and/or select it to retain in the population
//    - replace current population with selection

fn main() {
    let orig_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        orig_hook(panic_info);
        process::exit(-1);
    }));

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Lemurs",
        native_options,
        Box::new(|_| Box::new(LemursApp::new())),
    )
    .unwrap();
}
