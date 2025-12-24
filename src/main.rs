mod state;
mod client;

use std::env;

use wayland_client::Connection;

use crate::client::{build_state, build_surface, draw_plain};

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

    draw_plain(&state, &qh, &surface);

    println!("Wallpaper set! Press Ctrl+C to exit");

    loop {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
}

struct Config {
    image_path: String,
}

impl Config {
    fn build(args: Vec<String>) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err("Insufficient arguments provided");
        }
        let image_path = args[1].clone();
        let conf = Config { image_path, };
        Ok(conf)
    }
}

