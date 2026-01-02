use wayland_client::{Connection, Dispatch, QueueHandle, protocol::{wl_buffer, wl_compositor, wl_output, wl_registry, wl_shm, wl_shm_pool, wl_surface}};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};


pub struct AppState {
    pub compositor: Option<wl_compositor::WlCompositor>,
    pub layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    pub shm: Option<wl_shm::WlShm>, // Shared memory

    // Monitors
    pub outputs: Vec<wl_output::WlOutput>,

    pub wallpapers: Vec<Wallpaper>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            compositor: None,
            layer_shell: None,
            shm: None,
            outputs: Vec::new(),
            wallpapers: Vec::new(),
        }
    }
}

pub struct Wallpaper {
    pub surface: wl_surface::WlSurface,
    pub layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    pub width: u32,
    pub height: u32,
    pub configured: bool,
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
                proxy.ack_configure(serial);

                // Find which wallpaper this event belongs to and update it
                if let Some(wallpaper) = state.wallpapers.iter_mut().find(|w| w.layer_surface == *proxy) {
                    wallpaper.width = width;
                    wallpaper.height = height;
                    wallpaper.configured = true;
                    println!("Monitor configured: {}x{}", width, height);
                }
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

impl Dispatch<wl_output::WlOutput, ()> for AppState {
    fn event(_: &mut Self, _: &wl_output::WlOutput, _: wl_output::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
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
        if let wl_registry::Event::Global { name, interface, version: _ } = event {
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
                "wl_output" => {
                    let output = proxy.bind::<wl_output::WlOutput, _, _>(name, 4, qhandle, ());
                    state.outputs.push(output);
                },
                _ => {},
            }
        }
        
    }
    
}

