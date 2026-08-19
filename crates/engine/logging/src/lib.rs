//! Logging system for LITHOS

use std::sync::Once;
use tracing::{info, Level, Subscriber};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use lithos_engine_memory::GLOBAL_STATS;

static INIT: Once = Once::new();

/// Initialize logging system
pub fn init(config: LoggingConfig) {
    let level = config.default_level.clone();
    INIT.call_once(move || {
        let fmt_layer = fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .json();

        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(level.clone()));

        let subscriber = tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer);

        if config.enable_memory_logging {
            subscriber.with(MemoryLoggingLayer).init();
        } else {
            subscriber.init();
        }

        info!("Logging initialized with level: {}", level);
    });
}

/// Logging configuration
#[derive(Clone, Debug)]
pub struct LoggingConfig {
    pub default_level: String,
    pub enable_memory_logging: bool,
    pub enable_profiling: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            default_level: "info".to_string(),
            enable_memory_logging: true,
            enable_profiling: true,
        }
    }
}

/// Layer that logs memory stats periodically
struct MemoryLoggingLayer;

impl<S: Subscriber> Layer<S> for MemoryLoggingLayer {
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        // Periodically log memory stats
        if event.metadata().level() <= &Level::DEBUG {
            let stats = &GLOBAL_STATS;
            tracing::trace!(
                allocated = stats.current_bytes(),
                peak = stats.peak_bytes(),
                total_alloc = stats.total_allocated_bytes(),
                allocs = stats.allocation_count(),
                "Memory stats"
            );
        }
    }
}

/// In-game console for commands
pub mod console {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::io::{self, Write};

    pub type CommandFn = Box<dyn Fn(&[&str]) -> Result<String, String> + Send + Sync>;

    pub struct Console {
        commands: HashMap<String, CommandFn>,
        history: Vec<String>,
        history_index: usize,
        buffer: String,
    }

    impl Console {
        pub fn new() -> Self {
            Self {
                commands: HashMap::new(),
                history: Vec::new(),
                history_index: 0,
                buffer: String::new(),
            }
        }

        pub fn register(&mut self, name: &str, desc: &str, f: impl Fn(&[&str]) -> Result<String, String> + Send + Sync + 'static) {
            self.commands.insert(name.to_string(), Box::new(f));
            // Could store descriptions separately for help command
        }

        pub fn execute(&mut self, input: &str) -> Result<String, String> {
            let input = input.trim();
            if input.is_empty() {
                return Ok(String::new());
            }

            self.history.push(input.to_string());
            self.history_index = self.history.len();

            let parts: Vec<&str> = input.split_whitespace().collect();
            let cmd = parts[0];
            let args = &parts[1..];

            if let Some(f) = self.commands.get(cmd) {
                f(args)
            } else {
                Err(format!("Unknown command: {}", cmd))
            }
        }

        pub fn get_history(&self) -> &[String] {
            &self.history
        }

        pub fn complete(&self, partial: &str) -> Vec<String> {
            self.commands.keys()
                .filter(|k| k.starts_with(partial))
                .cloned()
                .collect()
        }
    }

    /// Global console instance
    static mut GLOBAL_CONSOLE: Option<Console> = None;

    pub fn init_global() {
        unsafe {
            GLOBAL_CONSOLE = Some(Console::new());
        }
    }

    pub fn global() -> &'static mut Console {
        unsafe {
            GLOBAL_CONSOLE.as_mut().expect("Console not initialized")
        }
    }

    pub fn execute(input: &str) -> Result<String, String> {
        global().execute(input)
    }
}

/// Structured logging macros
#[macro_export]
macro_rules! trace_span {
    ($name:expr) => {
        tracing::trace_span!($name)
    };
}

#[macro_export]
macro_rules! debug_span {
    ($name:expr) => {
        tracing::debug_span!($name)
    };
}

#[macro_export]
macro_rules! info_span {
    ($name:expr) => {
        tracing::info_span!($name)
    };
}

#[macro_export]
macro_rules! warn_span {
    ($name:expr) => {
        tracing::warn_span!($name)
    };
}

#[macro_export]
macro_rules! error_span {
    ($name:expr) => {
        tracing::error_span!($name)
    };
}