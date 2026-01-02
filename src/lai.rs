use std::fs;
use std::process::{Command, Stdio};
use std::path::Path;

use anyhow::{Context, Result};

pub fn generate_local(prompt: &str, output_path: &str) -> Result<()> {
    println!("Running local image generation...");

    let status = Command::new("nice")
        .arg("-n").arg("19") // lowest priority
        .arg("stable-diffusion")
        .arg("--prompt")
        .arg(prompt)
        .arg("--sd-version").arg("v1-5")
        .arg("--n-steps").arg("100")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to execute 'stable-diffusion' command. Is it in your PATH?")?;

    if !status.success() {
        anyhow::bail!("Local generation failed. Check the logs above.");
    }

    let default_output = Path::new("sd_final.png");
    
    if default_output.exists() {
        fs::rename(default_output, output_path)
            .context("Failed to move the generated image to the output path")?;
        println!("Local wallpaper saved to: {}", output_path);
    } else {
        anyhow::bail!("Success reported, but 'sd_final.png' was not found!");
    }

    Ok(())
}
