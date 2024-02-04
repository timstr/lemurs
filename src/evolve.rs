use std::fs::File;
use std::io::{stdin, Read, Write};
use std::process::{Command, Stdio};
use std::{env, fs, panic, process};

use std::sync::{
    mpsc::{channel, Sender},
    Arc,
};

use eframe::egui::PointerButton;
use eframe::{
    egui::{self, Context},
    epaint::{Color32, ColorImage, TextureHandle},
    App, Frame,
};
use lemurs::instruction::assemble;
use lemurs::machine::Machine;
use rand::{thread_rng, Rng};
use rustfft::{num_complex::Complex32, Fft, FftPlanner};

use threadpool::ThreadPool;

const OUTPUT_PREVIEW_LENGTH: usize = 65536 * 8 * 8;
const FFT_WINDOW_SIZE: usize = 256;
// const FFT_HOP_SIZE: usize = FFT_WINDOW_SIZE; // / 4;
// const FFT_HOP_SIZE: usize = 1920 / FFT_WINDOW_SIZE;
const FFT_HOP_SIZE: usize = FFT_WINDOW_SIZE * 8;

fn make_spectrogram_texture(
    program_output: &[u8],
    fft: &dyn Fft<f32>,
    window_coefficients: &[f32],
) -> ColorImage {
    let mut buffer: Vec<Complex32> = Vec::new();
    buffer.resize(FFT_WINDOW_SIZE, Complex32::default());
    assert!(program_output.len() >= FFT_WINDOW_SIZE);
    let image_height = FFT_WINDOW_SIZE / 2;
    let image_width = (program_output.len() - FFT_WINDOW_SIZE + FFT_HOP_SIZE) / FFT_HOP_SIZE;
    println!("image_width = {}", image_width);

    let mut pixels: Vec<Color32> = Vec::new();
    pixels.resize(image_width * image_height, Color32::BLACK);

    let mut abs_min = f32::MAX;
    let mut abs_max = f32::MIN;

    let colours = [
        (0.0, 0.0, 0.0),
        (0.0, 0.3, 0.8),
        (1.0, 0.5, 0.0),
        (1.0, 1.0, 1.0),
    ];

    let get_colour = |t: f32| -> Color32 {
        let i_f = t * (colours.len() - 1) as f32;
        let i_prev = i_f.floor() as usize;
        let i_next = i_f.ceil() as usize;
        let d = i_f.fract();
        let c_prev = colours[i_prev];
        let c_next = colours[i_next];
        let (r, g, b) = (
            c_prev.0 + d * (c_next.0 - c_prev.0),
            c_prev.1 + d * (c_next.1 - c_prev.1),
            c_prev.2 + d * (c_next.2 - c_prev.2),
        );
        Color32::from_rgb(
            (r * 255.0).clamp(0.0, 255.0) as u8,
            (g * 255.0).clamp(0.0, 255.0) as u8,
            (b * 255.0).clamp(0.0, 255.0) as u8,
        )
    };

    for h in 0..image_width {
        let output_offset = h * FFT_HOP_SIZE;
        for (i, v) in buffer.iter_mut().enumerate() {
            *v = Complex32 {
                re: program_output[output_offset + i] as f32 * window_coefficients[i],
                im: 0.0,
            };
        }

        fft.process(&mut buffer);

        let v_min: f32 = 1e0;
        let v_max: f32 = 1e4;
        let log_min = v_min.ln();
        let log_max = v_max.ln();
        let k = 1.0 / (log_max - log_min);
        for (i, v) in buffer[0..FFT_WINDOW_SIZE / 2].iter().enumerate() {
            let abs = v.norm();
            abs_min = abs_min.min(abs);
            abs_max = abs_max.max(abs);
            let log_abs = abs.clamp(v_min, v_max).ln();
            let t = (log_abs - log_min) * k;
            let px = h;
            let py = image_height - 1 - i;
            pixels[(py * image_width) + px] = get_colour(t)
        }
    }

    ColorImage {
        size: [image_width, image_height],
        pixels,
    }
}

struct AudioQueue {
    current_index: Option<usize>,
    sender: Sender<Vec<u8>>,
    _aplay_process: std::process::Child,
    _aplay_writer_thread: std::thread::JoinHandle<()>,
}

impl AudioQueue {
    fn new() -> AudioQueue {
        let (sender, receiver) = channel::<Vec<u8>>();
        let mut current_data: Option<Vec<u8>> = None;
        let mut current_data_index = 0;

        let channels: usize = 4;
        let sample_rate: usize = 64_000;
        let chunk_size = 4096;

        let mut aplay_process = Command::new("aplay")
            .args([
                format!("-c{}", channels),
                format!("-r{}", sample_rate),
                format!("--buffer-size={}", chunk_size * channels),
            ])
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();

        let mut aplay_stdin = aplay_process.stdin.take().unwrap();

        let chunk_interval =
            std::time::Duration::from_secs_f64(channels as f64 / sample_rate as f64);

        let mut timestamp = std::time::Instant::now();
        let mut empty_chunk: Vec<u8> = Vec::new();
        empty_chunk.resize(chunk_size, 0);
        let aplay_writer_thread = std::thread::spawn(move || loop {
            while let Ok(data) = receiver.try_recv() {
                current_data = Some(data);
                current_data_index = 0;
            }

            let Some(d) = &current_data else {
                aplay_stdin.write_all(&empty_chunk).unwrap();
                continue;
            };

            // for i in 0..chunk_size {
            //     let b = d.get(current_data_index + i).cloned().unwrap_or(0);
            //     aplay_stdin.write(&[b]).unwrap();
            // }
            let end_data_index = (current_data_index + chunk_size).min(d.len() - 1);
            aplay_stdin
                .write_all(&d[current_data_index..end_data_index])
                .unwrap();
            current_data_index += chunk_size;
            if current_data_index >= d.len() {
                current_data = None;
                current_data_index = 0;
            }

            let next_timestamp = timestamp + chunk_interval;
            std::thread::sleep(next_timestamp - std::time::Instant::now());
            timestamp = next_timestamp;
        });

        AudioQueue {
            current_index: None,
            sender,
            _aplay_process: aplay_process,
            _aplay_writer_thread: aplay_writer_thread,
        }
    }

    fn queue_audio(&mut self, index: usize, data: &[u8]) {
        if self.current_index != Some(index) {
            self.current_index = Some(index);
            self.sender.send(data.to_vec()).unwrap()
        }
    }
}

struct Instance {
    program: Vec<u8>,
    output: Vec<u8>,
    spectrogram_image: ColorImage,
    spectrogram_texture: Option<TextureHandle>,
    is_selected: bool,
}

impl Instance {
    fn new(program: Vec<u8>, fft: &dyn Fft<f32>, window_coefficients: &[f32]) -> Instance {
        let mut output = Vec::with_capacity(OUTPUT_PREVIEW_LENGTH);

        let mut machine = Machine::new(program.clone());

        let steps_per_iter = 2048;
        let max_iters: usize = 2048 * 8 * 8;

        for _ in 0..max_iters {
            machine.run(steps_per_iter, &mut output);
            if output.len() > OUTPUT_PREVIEW_LENGTH {
                break;
            }
        }

        while output.len() < OUTPUT_PREVIEW_LENGTH {
            output.push(0);
        }

        let spectrogram_image = make_spectrogram_texture(&output, fft, window_coefficients);

        Instance {
            program,
            output,
            spectrogram_image,
            spectrogram_texture: None,
            is_selected: false,
        }
    }
}

pub struct LemursApp {
    population: Vec<Instance>,
    fft: Arc<dyn Fft<f32>>,
    window_coefficients: Vec<f32>,
    mutation_amount: usize,
    desired_population_size: usize,
    audio_queue: AudioQueue,
    threadpool: ThreadPool,
}

fn random_program(length: usize) -> Vec<u8> {
    (0..length).map(|_| thread_rng().gen()).collect()
}

fn mutate_program(program: &mut Vec<u8>) {
    let mutation_type: u8 = thread_rng().gen_range(0..20);
    match mutation_type {
        0 => {
            // insert byte
            let i = thread_rng().gen_range(0..=program.len());
            let b: u8 = thread_rng().gen();
            program.insert(i, b);
        }
        1 => {
            // erase byte
            if program.len() <= 16 {
                // idk
                return;
            }
            let i = thread_rng().gen_range(0..program.len());
            program.remove(i);
        }
        2..=9 => {
            // randomize byte
            let i = thread_rng().gen_range(0..program.len());
            let b: u8 = thread_rng().gen();
            program[i] = b;
        }
        10.. => {
            // flip bit
            let i = thread_rng().gen_range(0..program.len());
            let b: u8 = 1 << thread_rng().gen_range(0..=7);
            program[i] ^= b;
        }
    }
}

impl LemursApp {
    pub fn new(initial_program: Vec<u8>) -> LemursApp {
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(FFT_WINDOW_SIZE);

        let k_inv_window_size = 1.0 / (FFT_WINDOW_SIZE as f32);
        let window_coefficients: Vec<f32> = (0..FFT_WINDOW_SIZE)
            .map(|i| {
                let t = (i as f32) * k_inv_window_size;
                0.5 - 0.5 * (t * std::f32::consts::TAU).cos()
            })
            .collect();

        let mut threadpool = ThreadPool::new(std::thread::available_parallelism().unwrap().into());

        let desired_population_size = 25;

        // TODO: add ThreadPool::iota or similar to avoid separate vec here
        let dummy_indices: Vec<usize> = (0..desired_population_size).collect();

        let population = threadpool.map(&dummy_indices, |_| {
            let mut p = initial_program.clone();
            for _ in 0..1 {
                mutate_program(&mut p);
            }
            Instance::new(p, &*fft, &window_coefficients)
        });

        LemursApp {
            population,
            fft,
            window_coefficients,
            mutation_amount: 8,
            desired_population_size,
            audio_queue: AudioQueue::new(),
            threadpool,
        }
    }

    fn show_instance(&mut self, ui: &mut egui::Ui, index: usize) {
        let instance = &mut self.population[index];
        let (background, border) = if instance.is_selected {
            (Color32::DARK_GREEN, Color32::GREEN)
        } else {
            (Color32::BLACK, Color32::GRAY)
        };
        let ir = egui::Frame::default()
            .stroke(egui::Stroke::new(2.0, border))
            .fill(background)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height());
                ui.vertical(|ui| {
                    // TODO: buttons to listen longer or save to disk?

                    let texture: &TextureHandle =
                        instance.spectrogram_texture.get_or_insert_with(|| {
                            ui.ctx().load_texture(
                                "texture",
                                instance.spectrogram_image.clone(),
                                Default::default(),
                            )
                        });

                    ui.image(texture.id(), ui.available_size());
                });
            });
        let r = ir.response.interact(egui::Sense::click());
        if instance.is_selected {
            ui.painter().rect_filled(
                ir.response.rect,
                egui::Rounding::none(),
                Color32::from_rgba_unmultiplied(0, 255, 0, 64),
            );
        }
        if r.clicked_by(PointerButton::Primary) {
            instance.is_selected = !instance.is_selected;
        }
        if r.clicked_by(PointerButton::Secondary) {
            let stamp: u32 = thread_rng().gen();
            let filename = format!("lemurs_instance_{}.bin", stamp);
            let mut file = File::create(&filename).unwrap();
            file.write_all(&instance.program).unwrap();
            println!("Saved program to {}", filename);
        }
        if r.hovered() {
            self.audio_queue.queue_audio(index, &instance.output);
            ui.painter().rect_filled(
                ir.response.rect,
                egui::Rounding::none(),
                Color32::from_white_alpha(16),
            );
        }
    }

    fn mutate(&mut self) {
        let selected_programs: Vec<&[u8]> = self
            .population
            .iter()
            .filter_map(|i| -> Option<&[u8]> {
                if i.is_selected {
                    Some(&i.program)
                } else {
                    None
                }
            })
            .collect();

        let mut new_programs: Vec<Vec<u8>> = Vec::new();

        new_programs.resize_with(self.desired_population_size, || {
            let mut p: Vec<u8>;
            if selected_programs.len() == 0 {
                let i = thread_rng().gen_range(0..self.population.len());
                p = self.population[i].program.clone();
            } else {
                let i = thread_rng().gen_range(0..selected_programs.len());
                p = selected_programs[i].to_vec();
            }
            for _ in 0..self.mutation_amount {
                mutate_program(&mut p);
            }
            p
        });

        let new_population: Vec<Instance> = self.threadpool.map(&new_programs, |p| {
            // TODO: consider adding ThreadPool::map_into to avoid clone here
            Instance::new(p.clone(), &*self.fft, &self.window_coefficients)
        });

        self.population = new_population;
    }
}

impl App for LemursApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                egui::Frame::default()
                    .fill(Color32::DARK_BLUE)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            if ui.button("MUTATE").clicked() {
                                self.mutate();
                            }
                            ui.separator();
                            ui.label("Mutation Amount");
                            ui.add(egui::Slider::new(&mut self.mutation_amount, 1..=32));
                            ui.separator();
                            ui.label("Population Size");
                            ui.add(egui::Slider::new(
                                &mut self.desired_population_size,
                                1..=128,
                            ));
                        });
                    });

                let num_instances = self.population.len();
                // let num_divisions = (num_instances as f64).sqrt().ceil() as usize;
                // let num_columns = num_divisions / 2;
                // let num_rows = num_divisions * 2;
                let num_rows = num_instances;
                let num_columns = 1;

                if num_instances == 0 {
                    ui.label("No instances");
                    return;
                }

                let col_width = ui.available_width() / num_columns as f32;
                let row_height = ui.available_height() / num_rows as f32;

                egui::Grid::new("grid")
                    .min_col_width(col_width)
                    .max_col_width(col_width)
                    .min_row_height(row_height)
                    .spacing(egui::Vec2::ZERO)
                    .show(ui, |ui| {
                        for i in 0..self.population.len() {
                            self.show_instance(ui, i);
                            if (i + 1) % num_columns == 0 {
                                ui.end_row();
                            }
                        }
                    });
            });
        });
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();

    // HACK for debugging
    // let args: Vec<String> = ["", "./example2.asm", "--assemble"]
    //     .iter()
    //     .map(|s| s.to_string())
    //     .collect();

    if args.len() > 3 {
        println!("Usage:");
        println!("  Evolve a random program:");
        println!("   {}", args[0]);
        println!("");
        println!("  Evolve a binary file:");
        println!("   {} path/to/file.bin", args[0]);
        println!("");
        println!("  Assemble a program and evolve it:");
        println!("   {} path/to/file.asm --assemble", args[0]);
        println!("");
        println!("  To receive a binary from stdin until EOF to evolve:");
        println!("   {} -", args[0]);
        println!("");
        return;
    }
    let mut memory = if args.len() == 1 {
        random_program(256)
    } else if args[1] == "-" {
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

    let orig_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        orig_hook(panic_info);
        process::exit(-1);
    }));

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Lemurs",
        native_options,
        Box::new(|_| Box::new(LemursApp::new(memory))),
    )
    .unwrap();
}
