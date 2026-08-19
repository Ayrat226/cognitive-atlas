//! LITHOS Server - Dedicated server entry point

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use clap::Parser;
use tokio::signal;
use tracing::{info, error, warn, debug};
use lithos_engine_config::{AppConfig, ConfigManager};
use lithos_engine_jobs::JobPool;
use lithos_engine_memory::GLOBAL_STATS;

#[derive(Parser, Debug)]
#[command(name = "lithos-server", version, about = "LITHOS Dedicated Server")]
struct Args {
    /// Config file path
    #[arg(short, long, default_value = "server.toml")]
    config: PathBuf,

    /// Number of worker threads
    #[arg(short, long)]
    threads: Option<usize>,

    /// Port to listen on
    #[arg(short, long)]
    port: Option<u16>,

    /// World save directory
    #[arg(long)]
    world_dir: Option<PathBuf>,

    /// Max players
    #[arg(long)]
    max_players: Option<u32>,

    /// Tick rate (Hz)
    #[arg(long)]
    tick_rate: Option<u32>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Load configuration
    let config_manager = ConfigManager::new(args.config.clone())?;
    let mut app_config = config_manager.get();

    // Apply command-line overrides
    if let Some(t) = args.threads {
        app_config.engine.jobs.num_threads = t;
    }
    if let Some(p) = args.port {
        app_config.network.server.port = p;
    }
    if let Some(w) = args.world_dir {
        app_config.persistence.save_directory = w;
    }
    if let Some(m) = args.max_players {
        app_config.network.server.max_players = m;
    }
    if let Some(t) = args.tick_rate {
        app_config.network.server.tick_rate = t;
        app_config.simulation.tick_rate = t;
    }

    info!("Starting LITHOS Server v{}", env!("CARGO_PKG_VERSION"));
    info!("Config: {:?}", args.config);
    info!("Threads: {}", app_config.engine.jobs.num_threads);
    info!("Port: {}", app_config.network.server.port);
    info!("Max players: {}", app_config.network.server.max_players);
    info!("Tick rate: {}", app_config.network.server.tick_rate);

    // Initialize job pool
    let job_pool = JobPool::new(app_config.engine.jobs.num_threads);

    // Initialize world (placeholder)
    info!("Initializing world...");
    // let world = World::new(&app_config.simulation.worldgen).await?;

    // Start server systems
    info!("Starting server systems...");
    // let network = NetworkServer::new(&app_config.network.server).await?;
    // let persistence = PersistenceManager::new(&app_config.persistence)?;

    // Main server loop
    info!("Server running on port {}", app_config.network.server.port);
    info!("Press Ctrl+C to stop");

    let tick_duration = Duration::from_secs_f64(1.0 / app_config.network.server.tick_rate as f64);
    let mut last_tick = Instant::now();
    let mut frame_count: u64 = 0;

    loop {
        let frame_start = Instant::now();

        // Check for shutdown signal
        if signal::ctrl_c().await.is_ok() {
            info!("Shutdown signal received");
            break;
        }

        // Run simulation tick
        let tick_start = Instant::now();
        // world.tick(tick_duration).await?;
        // network.tick().await?;
        let tick_time = tick_start.elapsed();

        // Run job pool tasks
        // job_pool.wait_idle();

        frame_count += 1;

        // Log stats periodically
        if frame_count % (app_config.network.server.tick_rate as u64 * 10) == 0 {
            let mem = GLOBAL_STATS.current_bytes();
            let peak = GLOBAL_STATS.peak_bytes();
            info!(
                "Tick: {}, Frame time: {:.2}ms, Tick time: {:.2}ms, Mem: {:.2}MB, Peak: {:.2}MB",
                frame_count,
                frame_start.elapsed().as_secs_f64() * 1000.0,
                tick_time.as_secs_f64() * 1000.0,
                mem as f64 / 1_048_576.0,
                peak as f64 / 1_048_576.0,
            );
        }

        // Sleep to maintain tick rate
        let elapsed = frame_start.elapsed();
        if elapsed < tick_duration {
            tokio::time::sleep(tick_duration - elapsed).await;
        } else {
            warn!("Tick overrun: {:.2}ms (budget: {:.2}ms)", elapsed.as_secs_f64() * 1000.0, tick_duration.as_secs_f64() * 1000.0);
        }

        last_tick = frame_start;
    }

    // Graceful shutdown
    info!("Shutting down server...");
    // network.shutdown().await?;
    // persistence.save_all().await?;
    job_pool.shutdown();

    info!("Server shutdown complete");
    Ok(())
}