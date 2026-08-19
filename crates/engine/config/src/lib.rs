//! Configuration system for LITHOS

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use config::{Config, File, Environment};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Config error: {0}")]
    Config(#[from] config::ConfigError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
}

/// Engine configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EngineConfig {
    pub memory: MemoryConfig,
    pub jobs: JobsConfig,
    pub logging: LoggingConfig,
    pub profiling: ProfilingConfig,
    pub window: WindowConfig,
    pub rendering: RenderingConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub arena_size_mb: usize,
    pub pool_block_sizes: Vec<usize>,
    pub enable_tracking: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JobsConfig {
    pub num_threads: usize,
    pub enable_profiling: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub enable_memory_logging: bool,
    pub enable_console: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProfilingConfig {
    pub enable_tracy: bool,
    pub frame_history: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub resizable: bool,
    pub samples: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderingConfig {
    pub max_frames_in_flight: u32,
    pub enable_validation: bool,
    pub enable_gpu_debug: bool,
    pub texture_quality: TextureQuality,
    pub shadow_quality: ShadowQuality,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum TextureQuality { Low, Medium, High, Ultra }

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ShadowQuality { Off, Low, Medium, High, Ultra }

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            memory: MemoryConfig {
                arena_size_mb: 64,
                pool_block_sizes: vec![64, 128, 256, 512, 1024, 2048],
                enable_tracking: true,
            },
            jobs: JobsConfig {
                num_threads: num_cpus::get().saturating_sub(1).max(1),
                enable_profiling: true,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                enable_memory_logging: true,
                enable_console: true,
            },
            profiling: ProfilingConfig {
                enable_tracy: true,
                frame_history: 120,
            },
            window: WindowConfig {
                title: "LITHOS".to_string(),
                width: 1920,
                height: 1080,
                fullscreen: false,
                vsync: true,
                resizable: true,
                samples: 4,
            },
            rendering: RenderingConfig {
                max_frames_in_flight: 3,
                enable_validation: cfg!(debug_assertions),
                enable_gpu_debug: cfg!(debug_assertions),
                texture_quality: TextureQuality::High,
                shadow_quality: ShadowQuality::High,
            },
        }
    }
}

/// Simulation configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub tick_rate: u32,
    pub max_substeps: u32,
    pub voxel: VoxelConfig,
    pub materials: MaterialsConfig,
    pub processes: ProcessesConfig,
    pub logistics: LogisticsConfig,
    pub aggregation: AggregationConfig,
    pub worldgen: WorldGenConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoxelConfig {
    pub chunk_size: u32,
    pub region_size: u32,
    pub max_loaded_chunks: usize,
    pub lod_distances: Vec<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialsConfig {
    pub max_batches: usize,
    pub thermal_diffusion_enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessesConfig {
    pub max_active_processes: usize,
    pub implicit_solve: bool,
    pub max_substeps: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogisticsConfig {
    pub fluid_solver_iterations: u32,
    pub power_solver_tolerance: f32,
    pub item_batch_size: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AggregationConfig {
    pub aggregate_distance: f32,
    pub disaggregate_distance: f32,
    pub update_interval_ticks: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldGenConfig {
    pub seed: u64,
    pub generator_version: u32,
    pub noise_octaves: u32,
    pub noise_persistence: f32,
    pub noise_lacunarity: f32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            tick_rate: 60,
            max_substeps: 4,
            voxel: VoxelConfig {
                chunk_size: 32,
                region_size: 512,
                max_loaded_chunks: 50000,
                lod_distances: vec![48.0, 96.0, 192.0, 384.0],
            },
            materials: MaterialsConfig {
                max_batches: 100000,
                thermal_diffusion_enabled: true,
            },
            processes: ProcessesConfig {
                max_active_processes: 10000,
                implicit_solve: true,
                max_substeps: 4,
            },
            logistics: LogisticsConfig {
                fluid_solver_iterations: 10,
                power_solver_tolerance: 1e-6,
                item_batch_size: 64,
            },
            aggregation: AggregationConfig {
                aggregate_distance: 200.0,
                disaggregate_distance: 150.0,
                update_interval_ticks: 60,
            },
            worldgen: WorldGenConfig {
                seed: 0,
                generator_version: 1,
                noise_octaves: 6,
                noise_persistence: 0.5,
                noise_lacunarity: 2.0,
            },
        }
    }
}

/// Network configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub server: ServerNetworkConfig,
    pub client: ClientNetworkConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerNetworkConfig {
    pub bind_address: String,
    pub port: u16,
    pub tick_rate: u32,
    pub max_players: u32,
    pub bandwidth_limit_kbps: u32,
    pub packet_loss_simulation: f32,
    pub latency_simulation_ms: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientNetworkConfig {
    pub server_address: String,
    pub port: u16,
    pub interpolation: bool,
    pub prediction: bool,
    pub reconnect_attempts: u32,
    pub reconnect_delay_ms: u32,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            server: ServerNetworkConfig {
                bind_address: "0.0.0.0".to_string(),
                port: 7777,
                tick_rate: 20,
                max_players: 50,
                bandwidth_limit_kbps: 50,
                packet_loss_simulation: 0.0,
                latency_simulation_ms: 0,
            },
            client: ClientNetworkConfig {
                server_address: "127.0.0.1".to_string(),
                port: 7777,
                interpolation: true,
                prediction: true,
                reconnect_attempts: 10,
                reconnect_delay_ms: 1000,
            },
        }
    }
}

/// Persistence configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistenceConfig {
    pub save_directory: PathBuf,
    pub auto_save_interval_secs: u32,
    pub max_save_files: u32,
    pub compression: CompressionType,
    pub verify_saves: bool,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum CompressionType { None, Lz4, Zstd }

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            save_directory: PathBuf::from("saves"),
            auto_save_interval_secs: 300,
            max_save_files: 10,
            compression: CompressionType::Lz4,
            verify_saves: true,
        }
    }
}

/// Complete application configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub engine: EngineConfig,
    pub simulation: SimulationConfig,
    pub network: NetworkConfig,
    pub persistence: PersistenceConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            engine: EngineConfig::default(),
            simulation: SimulationConfig::default(),
            network: NetworkConfig::default(),
            persistence: PersistenceConfig::default(),
        }
    }
}

/// Configuration manager
pub struct ConfigManager {
    config: RwLock<AppConfig>,
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new(config_path: PathBuf) -> Result<Self, ConfigError> {
        let config = Self::load(&config_path)?;
        Ok(Self {
            config: RwLock::new(config),
            config_path,
        })
    }

    fn load(path: &PathBuf) -> Result<AppConfig, ConfigError> {
        let mut builder = Config::builder();

        // Default values
        builder = builder.add_source(config::Config::try_from(AppConfig::default())?);

        // Config file
        if path.exists() {
            builder = builder.add_source(File::with_name(path.to_str().unwrap()).required(false));
        }

        // Environment variables (LITHOS_ prefix)
        builder = builder.add_source(Environment::with_prefix("LITHOS").separator("__"));

        let config = builder.build()?.try_deserialize()?;
        Ok(config)
    }

    pub fn get(&self) -> AppConfig {
        self.config.read().unwrap().clone()
    }

    pub fn get_engine(&self) -> EngineConfig {
        self.config.read().unwrap().engine.clone()
    }

    pub fn get_simulation(&self) -> SimulationConfig {
        self.config.read().unwrap().simulation.clone()
    }

    pub fn get_network(&self) -> NetworkConfig {
        self.config.read().unwrap().network.clone()
    }

    pub fn get_persistence(&self) -> PersistenceConfig {
        self.config.read().unwrap().persistence.clone()
    }

    pub fn set(&self, config: AppConfig) {
        *self.config.write().unwrap() = config;
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let config = self.config.read().unwrap();
        let toml = toml::to_string_pretty(&*config)?;
        std::fs::write(&self.config_path, toml)?;
        Ok(())
    }

    pub fn reload(&self) -> Result<(), ConfigError> {
        let config = Self::load(&self.config_path)?;
        *self.config.write().unwrap() = config;
        Ok(())
    }
}

/// Global config manager
static GLOBAL_CONFIG: std::sync::OnceLock<ConfigManager> = std::sync::OnceLock::new();

pub fn init_config(path: PathBuf) -> Result<(), ConfigError> {
    GLOBAL_CONFIG.set(ConfigManager::new(path)?).ok();
    Ok(())
}

pub fn config() -> &'static ConfigManager {
    GLOBAL_CONFIG.get().expect("Config not initialized")
}