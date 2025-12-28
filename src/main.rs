mod state;
mod client;

use std::{env, fs, path::PathBuf, process::exit, time::{Duration, Instant}};

use wayland_client::{Connection, EventQueue, QueueHandle, protocol::wl_surface::WlSurface};

use crate::{client::{build_state, build_surface, draw_plain, set_img}, state::AppState};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mode = parse_args(args).expect("Expected mode");
    let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();
    let mut state = build_state(&conn, &mut event_queue);
    build_surface(&mut state, &qh);

    println!("Surface created! Waiting for configuration...");

    loop {
        event_queue.blocking_dispatch(&mut state).unwrap();

        let all_ready = state.wallpapers.iter().all(|w| w.configured);
        if all_ready {
            break;
        }
    }

    println!("Configuration complete. Ready to draw background");
    let m_index = 1;

    match mode {
        Mode::PLAIN => draw_plain(&state, &qh, m_index),
        Mode::IMAGE(image) => set_img(&state, &qh, &image, m_index),
        Mode::GENERATED(prompt) => exit(1),
        Mode::MULTIPLE(path) => cycle_images(&path, &mut state, &qh, &mut event_queue, &conn, m_index),
    }

    println!("Wallpaper set! Press Ctrl+C to exit");

    loop {
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
    if args.iter().any(|x| x == "--cycle") {
        return Ok(Mode::MULTIPLE(args[1].clone()))
    }
    Ok(Mode::IMAGE(args[1].clone()))
}

fn get_images_from_dir(path: &str) -> Vec<PathBuf> {
    let mut images = Vec::new();
    
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if let Some(ext_str) = extension.to_str() {
                            match ext_str.to_lowercase().as_str() {
                                "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp" => {
                                    images.push(path);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
    
    images
}

fn cycle_images(path: &str, state: &mut AppState, qh: &QueueHandle<AppState>, event_queue: &mut EventQueue<AppState>, conn: &Connection, m_index: usize) {
    let images = get_images_from_dir(path);
    let mut curr_img_index = 0;
    let mut next_switch_time = Instant::now();
    let interval = Duration::from_secs(5);
    loop {
       let now = Instant::now(); 
       if now >= next_switch_time {
           let img_path = &images[curr_img_index].to_string_lossy().into_owned();
           println!("Switching to {img_path}");
           set_img(&state, &qh, &img_path, m_index);
           curr_img_index = (curr_img_index + 1) % images.len();
           next_switch_time = now + interval;
           let _ = conn.flush();
       }
        event_queue.dispatch_pending(state).unwrap();
        std::thread::sleep(Duration::from_millis(100));
    }
}
