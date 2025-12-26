mod state;
mod client;

use std::{env, process::exit, time::Instant};

use wayland_client::Connection;

use crate::client::{build_state, build_surface, draw_plain, set_img};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mode = parse_args(args).expect("Expected mode");
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

    match mode {
        Mode::PLAIN => draw_plain(&state, &qh, &surface),
        Mode::IMAGE(image) => set_img(&state, &qh, &surface, &image),
        Mode::GENERATED(prompt) => exit(1),
        Mode::MULTIPLE(path) => exit(1),
    }

    println!("Wallpaper set! Press Ctrl+C to exit");

    loop {
        let now = Instant::now();
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
}

enum Mode {
    PLAIN,IMAGE(String),GENERATED(String),MULTIPLE(String)
}

fn parse_args(args: Vec<String>) -> Result<Mode, &'static str> {
    if args.len() < 2 {
        return Err("Insufficient arguments provided");
    }
    if args.iter().any(|x| x == "--plain") {
        return Ok(Mode::PLAIN)
    }
    Ok(Mode::IMAGE(args[1].clone()))

}

