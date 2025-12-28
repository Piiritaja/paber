use std::{os::fd::AsFd, process};
use std::num::NonZeroUsize;

use image::imageops::FilterType;
use nix::sys::mman::{mmap, MapFlags, ProtFlags};
use wayland_client::{EventQueue, QueueHandle};
use std::ffi::CStr;
use wayland_client::{Connection, protocol::wl_shm};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};
use nix::{sys::{memfd::{MemFdCreateFlag, memfd_create}}, unistd::ftruncate}; 

use crate::state::{AppState};

pub fn build_state(conn: &Connection, event_queue: &mut EventQueue<AppState>) -> AppState {
    let qh = event_queue.handle();

    let display = conn.display();

    let _registry = display.get_registry(&qh, ());

    println!("Connected to Wayland! Asking for globals...");

    let mut state = AppState::new();

    event_queue.roundtrip(&mut state).unwrap();
    state
}

pub fn build_surface(state: &mut AppState, qh: &QueueHandle<AppState>) {
    if state.layer_shell.is_none() {
        eprintln!("Error: This compositor does not support 'wlr_layer_shell_v1'.");
        eprintln!("Are you running Hyprland, Sway, or another wlroots-based compositor?");
        eprintln!("(GNOME and KDE often do not support this protocol).");
        process::exit(1);
    }

    println!("Success! Environment supports wallpapers. Continuing...");

    let compositor = state.compositor.as_ref().expect("Compositor not found");
    let layer_shell = state.layer_shell.as_ref().expect("Layer Shell not found");

    let outputs = state.outputs.clone(); // Avoid borrowing issues while mutating state

    for output in outputs {
        let surface = compositor.create_surface(qh, ());

        let layer_surface = layer_shell.get_layer_surface(&surface,
            Some(&output), 
            zwlr_layer_shell_v1::Layer::Background,
            "paber".to_string(),
            &qh,
            ());

        // Anchor to all 4 edges
        layer_surface.set_anchor(
            zwlr_layer_surface_v1::Anchor::Top |
            zwlr_layer_surface_v1::Anchor::Bottom |
            zwlr_layer_surface_v1::Anchor::Left |
            zwlr_layer_surface_v1::Anchor::Right
        );

        // don't rearrange other windows
        layer_surface.set_exclusive_zone(-1);

        surface.commit();
        state.wallpapers.push(crate::state::Wallpaper {
            surface,
            layer_surface,
            width: 0, // Will be updated by configure event
            height: 0,
            configured: false,
        });
    }

}

pub fn draw_plain(state: &AppState, qh: &QueueHandle<AppState>, wp_index: usize) {
    let wallpaper = &state.wallpapers[wp_index];
    let shm = state.shm.as_ref().unwrap();
    let size = (wallpaper.width * wallpaper.height * 4) as usize;

    let length = NonZeroUsize::new(size).expect("Window size cannot be zero!");

    let fd = memfd_create(
        CStr::from_bytes_with_nul(b"rust-wallpaper\0").unwrap(), 
        MemFdCreateFlag::empty()
    ).unwrap();

    ftruncate(&fd, size as i64).unwrap();

    let ptr = unsafe { 
        mmap(
            None, 
            length, 
            ProtFlags::PROT_READ | ProtFlags::PROT_WRITE, 
            MapFlags::MAP_SHARED, 
            Some(&fd), 
            0
        ).unwrap() 
    };

    let canvas = unsafe {
        std::slice::from_raw_parts_mut(ptr as *mut u32, (wallpaper.width * wallpaper.height) as usize)
    };

    for pixel in canvas.iter_mut() {
        *pixel = 0xFFFFFFFF;
    }

    let pool = shm.create_pool(fd.as_fd(), size as i32, &qh, ());

    let buffer = pool.create_buffer(
        0, 
        wallpaper.width as i32, 
        wallpaper.height as i32, 
        (wallpaper.width * 4) as i32, 
        wl_shm::Format::Argb8888, 
        &qh, 
        ()
    );

    wallpaper.surface.attach(Some(&buffer), 0, 0);

    wallpaper.surface.damage(0, 0, wallpaper.width as i32, wallpaper.height as i32);

    wallpaper.surface.commit();
}

pub fn set_img(state: &AppState, qh: &QueueHandle<AppState>, image_path: &str, wp_index: usize) {
    let wallpaper = &state.wallpapers[wp_index];
    let shm = state.shm.as_ref().unwrap();
    let size = (wallpaper.width * wallpaper.height * 4) as usize;

    let length = NonZeroUsize::new(size).expect("Window size cannot be zero!");

    let fd = memfd_create(
        CStr::from_bytes_with_nul(b"rust-wallpaper\0").unwrap(), 
        MemFdCreateFlag::empty()
    ).unwrap();

    ftruncate(&fd, size as i64).unwrap();

    let ptr = unsafe { 
        mmap(
            None, 
            length, 
            ProtFlags::PROT_READ | ProtFlags::PROT_WRITE, 
            MapFlags::MAP_SHARED, 
            Some(&fd), 
            0
        ).unwrap() 
    };

    let canvas = unsafe {
        std::slice::from_raw_parts_mut(ptr as *mut u32, (wallpaper.width * wallpaper.height) as usize)
    };

    println!("Loading image...");
    let img = image::open(image_path).expect("Failed to open image file");
    let resized_img = img.resize_exact(wallpaper.width, wallpaper.height, FilterType::Triangle);

    let rgba_buffer = resized_img.to_rgba8();

    for (i, pixel) in rgba_buffer.pixels().enumerate() {
        let [r, g, b, a] = pixel.0;
        
        // Pack the 4 bytes into one u32
        // We shift bits to place them in A-R-G-B order for the u32 integer.
        // When written to memory, Little Endian flips them to B-G-R-A.
        canvas[i] = ((a as u32) << 24) | 
                    ((r as u32) << 16) | 
                    ((g as u32) << 8)  | 
                    (b as u32);
    }

    println!("Image drawn to buffer.");

    let pool = shm.create_pool(fd.as_fd(), size as i32, &qh, ());

    let buffer = pool.create_buffer(
        0, 
        wallpaper.width as i32, 
        wallpaper.height as i32, 
        (wallpaper.width * 4) as i32, 
        wl_shm::Format::Argb8888, 
        &qh, 
        ()
    );

    wallpaper.surface.attach(Some(&buffer), 0, 0);

    wallpaper.surface.damage(0, 0, wallpaper.width as i32, wallpaper.height as i32);

    wallpaper.surface.commit();
}
