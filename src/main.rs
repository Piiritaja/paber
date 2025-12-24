mod state;
mod client;

use std::{env, process::exit};

use wayland_client::Connection;

use crate::client::{build_state, build_surface, draw_plain, set_img};

fn main() {
    let args: Vec<String> = env::args().collect();
    let conf = Config::build(args).unwrap();
    let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();
    let mut state = build_state(&conn, &mut event_queue);
    let surface = build_surface(&state, &qh);

    println!("Surface created! Waiting for configuration...");

    while !state.configured {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }

    println!("Configuration complete. Ready to draw background");

    match conf.mode {
        Mode::PLAIN => draw_plain(&state, &qh, &surface),
        Mode::IMAGE => set_img(&state, &qh, &surface, &conf.image_arg),
        Mode::GENERATED => exit(1),
    }

    println!("Wallpaper set! Press Ctrl+C to exit");

    loop {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
}

enum Mode {
    PLAIN,IMAGE,GENERATED
}

struct Config {
    image_arg: String,
    mode: Mode,
}

impl Config {
    fn build(args: Vec<String>) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err("Insufficient arguments provided");
        }
        let mode_pos = args.iter().position(|x| x == "--plain");
        let mut img_arg_pos = 1;

        let mut mode = Mode::IMAGE;
        if mode_pos.is_some() {
            mode = Mode::PLAIN;
        }
        let image_arg = args[img_arg_pos].clone();
        let conf = Config { image_arg, mode };
        Ok(conf)
    }
}

