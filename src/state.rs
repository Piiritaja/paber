use wayland_client::{Connection, Dispatch, QueueHandle, protocol::{wl_buffer, wl_compositor, wl_registry, wl_shm, wl_shm_pool, wl_surface}};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};


pub struct AppState {
    pub compositor: Option<wl_compositor::WlCompositor>,
    pub layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    pub surface: Option<wl_surface::WlSurface>,
    pub shm: Option<wl_shm::WlShm>, // Shared memory
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

