mod state;
mod client;

use clap::Parser;

use std::{env, fs, path::PathBuf, process::exit, time::{Duration, Instant}, usize};

use wayland_client::{Connection, EventQueue, QueueHandle, protocol::wl_surface::WlSurface};

use crate::{client::{build_state, build_surface, draw_plain, set_img}, state::AppState};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Sets a plain wallpaper with a specified hex value
    #[arg(long)]
    plain: Option<String>,

    /// Sets the specified image as a wallpaper
    #[arg(long)]
    image: Option<String>,

    /// Generates an image with a specified promt and sets it as a wallpaper
    #[arg(long)]
    generated: Option<String>,

    /// Cycles through images in a specified directory
    #[arg(long)]
    cycle: Option<String>,

    /// Monitors to apply the wallpaper to
    #[arg(short, long)]
    monitors: Option<String>,

    /// Interval for the cycle mode in seconds
    #[arg(short, long)]
    interval: Option<u64>,
}

fn main() {
    let args = Args::parse();
    let mode = determine_mode(&args).expect("Expected mode");
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
            break; }
    }

    println!("Configuration complete. Ready to draw background");
    let monitors_to_apply = parse_monitors(&args);

    match &mode {
        Mode::PLAIN => monitors_to_apply.iter().for_each(|m_index| draw_plain(&state, &qh, m_index.to_owned())),
        Mode::IMAGE(image) => monitors_to_apply.iter().for_each(|m_index| set_img(&state, &qh, &image, m_index.to_owned())),
        Mode::GENERATED(prompt) => exit(1),
        Mode::CYCLE(path, interval) => cycle_images(&path, interval, &mut state, &qh, &mut event_queue, &conn, monitors_to_apply),
    }

    println!("Wallpaper set! Press Ctrl+C to exit");

    loop {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
}

fn parse_monitors(args: &Args) -> Vec<usize> {
    if args.monitors.is_some() {
        return args.monitors.clone().unwrap().split(",").map(|x| x.parse().expect("Not a number!")).collect();
    }
    vec![0]
}

enum Mode {
    PLAIN,IMAGE(String),GENERATED(String),CYCLE(String, Duration)
}

fn determine_mode(args: &Args) -> Result<Mode, String> {
    if args.plain.is_some() {
        return Ok(Mode::PLAIN);
    }
    if args.image.is_some() {
        return Ok(Mode::IMAGE(args.image.clone().unwrap()));
    }
    if args.cycle.is_some() {
        let mut interval = 60 * 60; // Every hour
        if args.interval.is_some() {
           interval = args.interval.unwrap(); 
        }
        return Ok(Mode::CYCLE(args.cycle.clone().unwrap(), Duration::new(interval, 0)));
    }
    if args.generated.is_some() {
        return Ok(Mode::GENERATED(args.generated.clone().unwrap()));
    }
    Err("no mode found".to_string())
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

fn cycle_images(path: &str, interval: &Duration, state: &mut AppState, qh: &QueueHandle<AppState>, event_queue: &mut EventQueue<AppState>, conn: &Connection, monitors: Vec<usize>) {
    let images = get_images_from_dir(path);
    let mut curr_img_index = 0;
    let mut next_switch_time = Instant::now();
    loop {
       let now = Instant::now(); 
       if now >= next_switch_time {
           let img_path = &images[curr_img_index].to_string_lossy().into_owned();
           println!("Switching to {img_path}");
           monitors.iter().for_each(|m_index| set_img(&state, &qh, &img_path, m_index.to_owned()));
           curr_img_index = (curr_img_index + 1) % images.len();
           next_switch_time = now + interval.to_owned();
           let _ = conn.flush();
       }
        event_queue.dispatch_pending(state).unwrap();
        std::thread::sleep(Duration::from_millis(100));
    }
}
