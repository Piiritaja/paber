# Paber

A dynamic wallpaper manager for Wayland compositors that supports static images, AI-generated wallpapers, and cycling through image directories.

## Features

- **Static Images**: Set any image as your wallpaper
- **AI-Generated Wallpapers**: Generate wallpapers using Google's Gemini AI or local generation
- **Image Cycling**: Automatically cycle through images in a directory
- **Multi-Monitor Support**: Apply wallpapers to specific monitors
- **Context-Aware Generation**: AI-generated wallpapers consider time of day and date

## Requirements

- Wayland compositor with `wlr-layer-shell` protocol support
- Rust toolchain (edition 2024)
- For Online AI generation: Google Gemini API key
- For Offline AI generation: stable-diffusion from installed from https://github.com/huggingface/candle

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd paber
```

2. Build the project:
```bash
cargo build --release
```

3. Set up environment variables:
```bash
export PABER_HOME=/path/to/paber/storage/  # For storing generated images
export GEMINI_API_KEY=your_api_key_here    # For online AI generation
```

## Usage

### Plain Color Wallpaper
```bash
paber --plain "#ff5733"
```

### Static Image
```bash
paber --image /path/to/image.png
```

### AI-Generated Wallpaper

Generate online using Google Gemini:
```bash
paber --generated --prompt "A serene mountain landscape at sunset"
```

Generate locally:
```bash
paber --generated --local --prompt "Abstract geometric patterns"
```

Without a prompt, the tool generates context-aware wallpapers based on your username, time of day, and current date.

### Cycle Through Images
```bash
paber --cycle /path/to/images/directory --interval 3600
```

The `--interval` flag specifies seconds between changes (default: 3600 = 1 hour).

### Multi-Monitor Support
```bash
paber --image wallpaper.png --monitors 0,1,2
```

Apply wallpapers to specific monitors by their index (comma-separated).

## Supported Image Formats

- JPEG/JPG
- PNG
- WebP
- GIF
- BMP

## Architecture

The project consists of several modules:

- `main.rs:1` - CLI argument parsing and mode selection
- `client.rs:1` - Wayland client implementation and surface management
- `state.rs:1` - Application state and wallpaper configuration
- `gai.rs:1` - Google AI (Gemini) integration for online generation
- `lai.rs:1` - Local AI image generation

## Dependencies

Key dependencies include:
- `wayland-client` - Wayland protocol bindings
- `wayland-protocols-wlr` - wlr-layer-shell protocol support
- `image` - Image processing
- `clap` - Command-line argument parsing
- `reqwest` - HTTP client for API calls
- `serde`/`serde_json` - Serialization

## License

This project is licensed under the GNU General Public License v3.0 - see the LICENSE file for details.
