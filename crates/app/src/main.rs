//! LITHOS - Main application entry point

use std::path::PathBuf;
use clap::Parser;
use tracing::{info, error, warn};
use lithos_engine_config::{AppConfig, ConfigManager};
use lithos_engine_window::{run_app, EngineConfig};

#[derive(Parser, Debug)]
#[command(name = "lithos", version, about = "LITHOS - Voxel Planetary Simulation")]
struct Args {
    /// Config file path
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    /// Enable validation layers
    #[arg(long)]
    validation: bool,

    /// Override window width
    #[arg(long)]
    width: Option<u32>,

    /// Override window height
    #[arg(long)]
    height: Option<u32>,

    /// Fullscreen mode
    #[arg(long)]
    fullscreen: bool,

    /// Disable vsync
    #[arg(long)]
    no_vsync: bool,

    /// Headless mode (server only)
    #[arg(long)]
    headless: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Load configuration
    let config_manager = ConfigManager::new(args.config.clone())?;
    let mut app_config = config_manager.get();

    // Apply command-line overrides
    if args.validation {
        app_config.engine.rendering.enable_validation = true;
    }
    if let Some(w) = args.width {
        app_config.engine.window.width = w;
    }
    if let Some(h) = args.height {
        app_config.engine.window.height = h;
    }
    if args.fullscreen {
        app_config.engine.window.fullscreen = true;
    }
    if args.no_vsync {
        app_config.engine.window.vsync = false;
    }

    info!("Starting LITHOS v{}", env!("CARGO_PKG_VERSION"));
    info!("Config: {:?}", args.config);

    if args.headless {
        info!("Headless mode - not implemented yet");
        return Ok(());
    }

    // Run the application
    run_app(app_config.engine)?;

    info!("LITHOS shutdown complete");
    Ok(())
}