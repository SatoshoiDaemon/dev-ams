use crate::config::LoggingConfig;
use serde::{Deserialize, Serialize};
use std::{
    backtrace::Backtrace,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    panic,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl LogLevel {
    fn label(self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
            Self::Fatal => "FATAL",
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiagnosticContext {
    pub phase: Option<String>,
    pub mod_id: Option<String>,
    pub file: Option<String>,
    pub save: Option<String>,
    pub entity_id: Option<String>,
    pub action_id: Option<String>,
    pub event: Option<String>,
}

#[derive(Debug)]
struct LogFiles {
    latest: File,
    history: File,
}

#[derive(Clone, Debug)]
pub struct Logger {
    files: Arc<Mutex<LogFiles>>,
    context: Arc<Mutex<DiagnosticContext>>,
    pub path: PathBuf,
    pub history_path: PathBuf,
    root: PathBuf,
    minimum: LogLevel,
}

impl Logger {
    pub fn open(dir: &Path) -> io::Result<Self> {
        Self::open_with_config(dir, &LoggingConfig::default())
    }

    pub fn open_with_config(dir: &Path, config: &LoggingConfig) -> io::Result<Self> {
        fs::create_dir_all(dir)?;
        fs::create_dir_all(dir.join("crashes"))?;
        retain(
            dir,
            "session-",
            ".log",
            config.keep_sessions.map(|value| value.saturating_sub(1)),
        )?;
        retain(
            &dir.join("crashes"),
            "crash-",
            ".txt",
            config.keep_crash_reports,
        )?;
        let path = dir.join("latest.log");
        let history_path = unique_path(dir, "session-", ".log");
        let latest = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&path)?;
        let history = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&history_path)?;
        Ok(Self {
            files: Arc::new(Mutex::new(LogFiles { latest, history })),
            context: Arc::new(Mutex::new(DiagnosticContext::default())),
            path,
            history_path,
            root: dir.to_path_buf(),
            minimum: parse_level(&config.level),
        })
    }

    pub fn set_context(&self, context: DiagnosticContext) -> io::Result<()> {
        *self
            .context
            .lock()
            .map_err(|_| io::Error::other("diagnostic context lock poisoned"))? = context;
        Ok(())
    }

    pub fn context(&self) -> DiagnosticContext {
        self.context
            .lock()
            .map(|value| value.clone())
            .unwrap_or_default()
    }

    pub fn write(&self, level: LogLevel, target: &str, message: &str) -> io::Result<()> {
        if level < self.minimum {
            return Ok(());
        }
        let stamp = timestamp();
        let context_text = context_fields(&self.context());
        let line = if context_text.is_empty() {
            format!("[{stamp}] [{}] [{target}] {message}\n", level.label())
        } else {
            format!(
                "[{stamp}] [{}] [{target}] [{context_text}] {message}\n",
                level.label()
            )
        };
        let mut files = self
            .files
            .lock()
            .map_err(|_| io::Error::other("logger lock poisoned"))?;
        files.latest.write_all(line.as_bytes())?;
        files.latest.flush()?;
        files.history.write_all(line.as_bytes())?;
        files.history.flush()
    }

    pub fn log(&self, level: &str, message: &str) -> io::Result<()> {
        self.write(parse_level(level), "runtime", message)
    }
    pub fn trace(&self, target: &str, message: &str) -> io::Result<()> {
        self.write(LogLevel::Trace, target, message)
    }
    pub fn debug(&self, target: &str, message: &str) -> io::Result<()> {
        self.write(LogLevel::Debug, target, message)
    }
    pub fn info(&self, message: &str) -> io::Result<()> {
        self.write(LogLevel::Info, "runtime", message)
    }
    pub fn warn(&self, message: &str) -> io::Result<()> {
        self.write(LogLevel::Warn, "runtime", message)
    }
    pub fn error(&self, message: &str) -> io::Result<()> {
        self.write(LogLevel::Error, "runtime", message)
    }
    pub fn fatal(&self, message: &str) -> io::Result<()> {
        self.write(LogLevel::Fatal, "runtime", message)
    }

    pub fn write_crash_report(&self, message: &str) -> io::Result<PathBuf> {
        let crashes = self.root.join("crashes");
        fs::create_dir_all(&crashes)?;
        let path = unique_path(&crashes, "crash-", ".txt");
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)?;
        writeln!(file, "A MAGIC SOVEREIGN CRASH REPORT")?;
        writeln!(file, "Version: {}", env!("CARGO_PKG_VERSION"))?;
        writeln!(
            file,
            "Platform: {}-{}",
            std::env::consts::OS,
            std::env::consts::ARCH
        )?;
        writeln!(file, "Timestamp: {}", timestamp())?;
        writeln!(file, "Context: {:#?}", self.context())?;
        writeln!(file, "Error: {message}")?;
        writeln!(file, "Backtrace:\n{}", Backtrace::force_capture())?;
        Ok(path)
    }
}

pub fn install_panic_hook(logger: Logger) {
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |information| {
        let message = information.to_string();
        match logger.write_crash_report(&message) {
            Ok(path) => eprintln!(
                "The game encountered a fatal error.\nCrash report: {}\nRuntime log: {}",
                path.display(),
                logger.path.display()
            ),
            Err(error) => eprintln!(
                "The game encountered a fatal error: {message}\nFailed to write crash report: {error}"
            ),
        }
        previous(information);
    }));
}

fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn unique_path(dir: &Path, prefix: &str, suffix: &str) -> PathBuf {
    let base = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    for sequence in 0_u32.. {
        let path = dir.join(format!("{prefix}{base}-{sequence:03}{suffix}"));
        if !path.exists() {
            return path;
        }
    }
    unreachable!("the sequence space is practically unbounded")
}

fn parse_level(value: &str) -> LogLevel {
    match value.to_ascii_lowercase().as_str() {
        "trace" => LogLevel::Trace,
        "debug" => LogLevel::Debug,
        "warn" => LogLevel::Warn,
        "error" => LogLevel::Error,
        "fatal" => LogLevel::Fatal,
        _ => LogLevel::Info,
    }
}

fn context_fields(context: &DiagnosticContext) -> String {
    [
        ("phase", context.phase.as_deref()),
        ("mod", context.mod_id.as_deref()),
        ("file", context.file.as_deref()),
        ("save", context.save.as_deref()),
        ("entity", context.entity_id.as_deref()),
        ("action", context.action_id.as_deref()),
        ("event", context.event.as_deref()),
    ]
    .into_iter()
    .filter_map(|(key, value)| value.map(|value| format!("{key}={value}")))
    .collect::<Vec<_>>()
    .join(" ")
}

fn retain(dir: &Path, prefix: &str, suffix: &str, keep: Option<usize>) -> io::Result<()> {
    let Some(keep) = keep else {
        return Ok(());
    };
    let mut paths = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(prefix) && name.ends_with(suffix))
        })
        .collect::<Vec<_>>();
    paths.sort();
    let remove_count = paths.len().saturating_sub(keep);
    for path in paths.into_iter().take(remove_count) {
        fs::remove_file(path)?;
    }
    Ok(())
}
