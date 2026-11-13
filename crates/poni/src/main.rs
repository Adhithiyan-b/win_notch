//! Dynamic Notch — a living notch overlay for Windows 10/11.
//!
//! Architecture:
//! - One UI thread owns a layered top-most window, the Direct2D renderer and
//!   [`state::AppState`]. Rendering only happens while something moves.
//! - Services (media, lyrics, audio, power, Bluetooth, …) run on
//!   their own threads/callbacks and talk to the UI thread through
//!   [`events::EventSink`] (an mpsc channel + a posted window message).
//! - Pages draw with an immediate-mode [`ui::Ui`] and request side effects by
//!   pushing [`commands::Command`]s, which the app executes after each frame.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
mod plog;
mod activities;
mod app;
mod autostart;
mod clock;
mod commands;
mod dragdrop;
mod events;
mod gfx;
mod glass;
mod icons;
mod island;
mod last_call;
mod oyen;
mod pages;
mod pill;
mod paths;
mod png;
mod selftest;
mod services;
mod settings_host;
mod settings_window;
mod shell;
mod state;
mod tray;
mod ui;
mod version;
mod updates;
mod window;

fn main() {
    std::process::exit(app::main_entry());
}
