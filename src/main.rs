use std::{env, os::fd::AsFd, process::{self, exit}};

use std::num::NonZeroUsize;
use nix::{sys::{memfd::{MemFdCreateFlag, memfd_create}}, unistd::ftruncate}; 
use nix::sys::mman::{mmap, MapFlags, ProtFlags};
use std::ffi::CStr;

use wayland_client::{
    Connection, Dispatch, QueueHandle, globals::Global, protocol::{wl_buffer, wl_compositor, wl_registry, wl_shm, wl_shm_pool, wl_surface}
};

use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1,
    zwlr_layer_surface_v1,
};

struct AppState {
    compositor: Option<wl_compositor::WlCompositor>,
    layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    shm: Option<wl_shm::WlShm>, // Shared memory
    width: u32,
    height: u32,
    configured: bool,
}

impl Dispatch<wl_compositor::WlCompositor, ()> for AppState {
    fn event(
            _state: &mut Self,
            _proxy: &wl_compositor::WlCompositor,
            _event: <wl_compositor::WlCompositor as wayland_client::Proxy>::Event,
            _data: &(),
            _conn: &Connection,
            _qhandle: &QueueHandle<Self>,
        ) {
        
    }
}

impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for AppState {
    fn event(
            _state: &mut Self,
            _proxy: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
            _event: <zwlr_layer_shell_v1::ZwlrLayerShellV1 as wayland_client::Proxy>::Event,
            _data: &(),
            _conn: &Connection,
            _qhandle: &QueueHandle<Self>,
        ) {
        
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for AppState {
    fn event(
            _state: &mut Self,
            _proxy: &wl_surface::WlSurface,
            _event: <wl_surface::WlSurface as wayland_client::Proxy>::Event,
            _data: &(),
            _conn: &Connection,
            _qhandle: &QueueHandle<Self>,
        ) {
        
    }
    
}

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for AppState {
    fn event(
            state: &mut Self,
            proxy: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
            event: <zwlr_layer_surface_v1::ZwlrLayerSurfaceV1 as wayland_client::Proxy>::Event,
            _data: &(),
            _conn: &Connection,
            _qhandle: &QueueHandle<Self>,
        ) {
        if let zwlr_layer_surface_v1::Event::Configure { serial, width, height } = event {
            println!("Surface size is {}x{}", width, height);
            state.width = width;
            state.height = height; 
            state.configured = true;

            proxy.ack_configure(serial);
        }
    }
}

impl Dispatch<wl_shm::WlShm, ()> for AppState {
    fn event(
            _state: &mut Self,
            _proxy: &wl_shm::WlShm,
            _event: <wl_shm::WlShm as wayland_client::Proxy>::Event,
            _data: &(),
            _conn: &Connection,
            _qhandle: &QueueHandle<Self>,
        ) {
        
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for AppState {
    fn event(
            _state: &mut Self,
            _proxy: &wl_shm_pool::WlShmPool,
            _event: <wl_shm_pool::WlShmPool as wayland_client::Proxy>::Event,
            _data: &(),
            _conn: &Connection,
            _qhandle: &QueueHandle<Self>,
        ) {
        
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for AppState {
    fn event(
            state: &mut Self,
            proxy: &wl_registry::WlRegistry,
            event: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
            _data: &(),
            _conn: &Connection,
            qhandle: &QueueHandle<Self>,
        ) {
        if let wl_registry::Event::Global { name, interface, version } = event {
            match interface.as_str() {
                "wl_compositor" => {
                    let compositor = proxy.bind::<wl_compositor::WlCompositor, _, _>(
                        name,
                        4, // (version 4 is standard/safe)
                        qhandle,
                        ()
                    );
                    println!("Bound global: wl_compositor");
                    state.compositor = Some(compositor);
                }
                "zwlr_layer_shell_v1" => {
                    let layer_shell = proxy.bind::<zwlr_layer_shell_v1::ZwlrLayerShellV1, _, _>(
                        name,
                        1,
                        qhandle,
                        ()
                    );
                    println!("Bound global: zwlr_layer_shell_v1");
                    state.layer_shell = Some(layer_shell);
                },
                "wl_shm" => { state.shm = Some(proxy.bind(name, 1, qhandle, ())); },
                _ => {}
            }
        }
        
    }
    
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let conf = Config::build(args).unwrap();
    connect_wl();
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

fn connect_wl() {
    let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    let display = conn.display();

    let _registry = display.get_registry(&qh, ());

    println!("Connected to Wayland! Asking for globals...");

    let mut state = AppState {
        compositor: None,
        layer_shell: None,
        shm: None,
        width: 0,
        height: 0,
        configured: false,
    };

    event_queue.roundtrip(&mut state).unwrap();

    if state.layer_shell.is_none() {
        eprintln!("Error: This compositor does not support 'wlr_layer_shell_v1'.");
        eprintln!("Are you running Hyprland, Sway, or another wlroots-based compositor?");
        eprintln!("(GNOME and KDE often do not support this protocol).");
        process::exit(1);
    }

    println!("Success! Environment supports wallpapers. Continuing...");

    let compositor = state.compositor.as_ref().expect("Compositor not found");
    let layer_shell = state.layer_shell.as_ref().expect("Layer Shell not found");

    // Create plain surface
    let surface = compositor.create_surface(&qh, ());

    // Assign Background surface role
    let layer_surface = layer_shell.get_layer_surface(&surface,
        None, // Default monitor
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

    println!("Surface created! Waiting for configuration...");

    while !state.configured {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }

    println!("Configuration complete. Ready to draw background");

    let shm = state.shm.as_ref().unwrap();
    let size = (state.width * state.height * 4) as usize;

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
        std::slice::from_raw_parts_mut(ptr as *mut u32, (state.width * state.height) as usize)
    };

    for pixel in canvas.iter_mut() {
        *pixel = 0xFFFFFFFF;
    }

    let pool = shm.create_pool(fd.as_fd(), size as i32, &qh, ());

    let buffer = pool.create_buffer(
        0, 
        state.width as i32, 
        state.height as i32, 
        (state.width * 4) as i32, 
        wl_shm::Format::Argb8888, 
        &qh, 
        ()
    );

    surface.attach(Some(&buffer), 0, 0);

    surface.damage(0, 0, state.width as i32, state.height as i32);

    surface.commit();

    println!("Wallpaper set! Press Ctrl+C to exit");

    loop {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
}

impl Dispatch<wl_buffer::WlBuffer, ()> for AppState {
    fn event(
            _state: &mut Self,
            _proxy: &wl_buffer::WlBuffer,
            _event: <wl_buffer::WlBuffer as wayland_client::Proxy>::Event,
            _data: &(),
            _conn: &Connection,
            _qhandle: &QueueHandle<Self>,
        ) {
        
    }
}
