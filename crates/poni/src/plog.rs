//! Tiny file logger: %LOCALAPPDATA%\Poni\poni.log (rotated at 1 MB).

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

static LOG_SENDER: Mutex<Option<std::sync::mpsc::SyncSender<String>>> = Mutex::new(None);

pub fn init(path: PathBuf) {
    if let Ok(meta) = fs::metadata(&path) {
        if meta.len() > 1_000_000 {
            let _ = fs::rename(&path, path.with_extension("old.log"));
        }
    }
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let (tx, rx) = std::sync::mpsc::sync_channel::<String>(2048);
    let log_path = path;
    std::thread::Builder::new()
        .name("poni_logger".into())
        .spawn(move || {
            let mut file = OpenOptions::new().create(true).append(true).open(&log_path).ok();
            while let Ok(line) = rx.recv() {
                if let Some(f) = file.as_mut() {
                    let _ = f.write_all(line.as_bytes());
                } else if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&log_path) {
                    let _ = f.write_all(line.as_bytes());
                    file = Some(f);
                }
            }
        })
        .ok();
    if let Ok(mut g) = LOG_SENDER.lock() {
        *g = Some(tx);
    }
}

pub fn write(level: &str, msg: &str) {
    let t = crate::clock::local_now();
    let line = format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} [{level}] {msg}\n",
        t.year, t.month, t.day, t.hour, t.minute, t.second
    );
    if cfg!(debug_assertions) {
        eprint!("{line}");
    }
    if let Ok(g) = LOG_SENDER.lock() {
        if let Some(sender) = g.as_ref() {
            let _ = sender.try_send(line);
        }
    }
}

#[macro_export]
macro_rules! pinfo {
    ($($t:tt)*) => { $crate::plog::write("INFO", &format!($($t)*)) };
}

#[macro_export]
macro_rules! pwarn {
    ($($t:tt)*) => { $crate::plog::write("WARN", &format!($($t)*)) };
}

#[macro_export]
macro_rules! perror {
    ($($t:tt)*) => { $crate::plog::write("ERROR", &format!($($t)*)) };
}
