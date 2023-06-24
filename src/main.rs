use std::{env, process};

use cpu::CPU;

mod audio;
mod cpu;
mod input;
mod video;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Missing rom file");
        process::exit(1)
    }

    let rom_path = &args[1];

    let sdl = sdl2::init().unwrap();

    let mut cpu = CPU::new(&sdl, &rom_path);
    while cpu.tick() {}

    println!("Closed.")
}
