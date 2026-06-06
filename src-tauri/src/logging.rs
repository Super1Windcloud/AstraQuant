use log::{Level, LevelFilter};
use std::sync::Mutex;
use tauri_plugin_log::{Target, TargetKind};

#[derive(Debug, Clone)]
struct PendingLogRecord {
    level: Level,
    target: String,
    message: String,
    module_path: Option<String>,
    file: Option<String>,
    line: Option<u32>,
    count: usize,
}

#[derive(Default)]
struct DedupState {
    pending: Option<PendingLogRecord>,
}

pub(crate) struct DedupLogger {
    inner: Box<dyn log::Log>,
    state: Mutex<DedupState>,
}

impl DedupLogger {
    pub(crate) fn new(inner: Box<dyn log::Log>) -> Self {
        Self {
            inner,
            state: Mutex::new(DedupState::default()),
        }
    }

    fn emit_record(&self, pending: PendingLogRecord) {
        let rendered_message = if pending.count > 1 {
            format!("{} ({})", pending.message, pending.count)
        } else {
            pending.message
        };

        let mut builder = log::Record::builder();
        let args = format_args!("{rendered_message}");
        builder
            .args(args)
            .level(pending.level)
            .target(&pending.target)
            .module_path(pending.module_path.as_deref())
            .file(pending.file.as_deref())
            .line(pending.line);

        self.inner.log(&builder.build());
    }
}

impl log::Log for DedupLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        self.inner.enabled(metadata)
    }

    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let next = PendingLogRecord {
            level: record.level(),
            target: record.target().to_string(),
            message: record.args().to_string(),
            module_path: record.module_path().map(ToString::to_string),
            file: record.file().map(ToString::to_string),
            line: record.line(),
            count: 1,
        };

        let mut state = self
            .state
            .lock()
            .expect("dedup logger mutex poisoned while logging");

        match state.pending.as_mut() {
            Some(pending)
                if pending.level == next.level
                    && pending.target == next.target
                    && pending.message == next.message =>
            {
                pending.count += 1;
            }
            Some(_) => {
                let previous = state.pending.replace(next);
                drop(state);

                if let Some(previous) = previous {
                    self.emit_record(previous);
                }
            }
            None => {
                state.pending = Some(next);
            }
        }
    }

    fn flush(&self) {
        let pending = {
            let mut state = self
                .state
                .lock()
                .expect("dedup logger mutex poisoned while flushing");
            state.pending.take()
        };

        if let Some(pending) = pending {
            self.emit_record(pending);
        }

        self.inner.flush();
    }
}

fn ansi_color_for_level(level: Level) -> &'static str {
    match level {
        Level::Trace => "\x1b[90m",
        Level::Debug => "\x1b[36m",
        Level::Info => "\x1b[32m",
        Level::Warn => "\x1b[33m",
        Level::Error => "\x1b[31m",
    }
}

pub(crate) fn build_log_plugin() -> tauri_plugin_log::Builder {
    tauri_plugin_log::Builder::new()
        .level(LevelFilter::Debug)
        .level_for("reqwest", LevelFilter::Warn)
        .level_for("hyper", LevelFilter::Warn)
        .level_for("tao", LevelFilter::Warn)
        .targets([
            Target::new(TargetKind::Stdout).format(|out, message, record| {
                let color = ansi_color_for_level(record.level());
                out.finish(format_args!("{color}{message}\x1b[0m"))
            }),
            Target::new(TargetKind::LogDir {
                file_name: Some("backend".into()),
            }),
            Target::new(TargetKind::Webview),
        ])
}
