//! Background services. Each one reports to the UI thread through an
//! [`EventSink`]; a failing service is logged and skipped, never fatal.

pub mod addons;
pub mod audio;
pub mod backdrop;
pub mod bluetooth;
pub mod clipboard;
pub mod frames;
pub mod fullscreen;
pub mod http;
pub mod imaging;
pub mod lyrics;
pub mod media;
pub mod phonedrop;
pub mod power;
pub mod psu;
pub mod recorder;
pub mod sound;
pub mod notifications;
pub mod sysmon;
pub mod sysstats;

use crate::commands::MediaCmd;
use crate::events::EventSink;
use crate::state::MediaSession;
use poni_core::geom::Color;
use poni_core::settings::Settings;
use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};

pub struct Services {
    media: Option<media::MediaService>,
    lyrics: Option<lyrics::LyricsService>,
    _audio: Option<audio::AudioService>,
    _bluetooth: Option<bluetooth::BluetoothService>,
    fullscreen: Option<fullscreen::FullscreenWatcher>,
    addons: Option<addons::AddonService>,
    sink: EventSink,
}

fn started<T, E: std::fmt::Display>(name: &str, r: Result<T, E>) -> Option<T> {
    match r {
        Ok(v) => {
            crate::pinfo!("service {name}: started");
            Some(v)
        }
        Err(e) => {
            crate::pwarn!("service {name}: unavailable ({e})");
            None
        }
    }
}

impl Services {
    pub fn start(sink: &EventSink, _island: HWND, helper: HWND, settings: &Settings) -> Self {
        let media = started("media", media::MediaService::start(sink.clone()));
        let lyrics = started("lyrics", lyrics::LyricsService::start(sink.clone()));
        let audio = started("audio", audio::AudioService::start(sink.clone()));
        let bluetooth = started("bluetooth", bluetooth::BluetoothService::start(sink.clone()));
        let fullscreen = started("fullscreen", fullscreen::FullscreenWatcher::start(helper, sink.clone()));
        let store = crate::autostart::packaged();
        let on = poni_core::addons::ALL
            .iter()
            .filter(|a| settings.addon_on(a.id) && !(store && a.id == poni_core::addons::DOWNLOADER.id))
            .map(|a| a.id)
            .collect();
        let addons = started("add-ons", addons::AddonService::start(sink.clone(), on));
        Self { media, lyrics, _audio: audio, _bluetooth: bluetooth, fullscreen, addons, sink: sink.clone() }
    }

    pub fn stop(self, _helper: HWND) {
        if let Some(f) = self.fullscreen {
            f.stop();
        }
    }

    /// Run a one-off download on the add-on thread (the update file).
    /// False when that thread couldn't start.
    pub fn run_job(&self, job: addons::Job) -> bool {
        self.addons.as_ref().map(|a| a.run(job)).is_some()
    }

    /// An add-on was switched on (download its files) or off (delete them).
    /// A Studio Pro add-on's files come only with Pro (`set_pro`).
    pub fn set_addon(&self, id: &'static str, on: bool) {
        if let Some(a) = &self.addons {
            a.set(id, on);
        }
    }

    /// Whether the account has Studio Pro: the Pro add-ons' files come with
    /// it, and stay as they are without it.
    pub fn set_pro(&self, pro: bool) {
        if let Some(a) = &self.addons {
            a.set_pro(pro);
        }
    }

    /// Step aside on (the band along the top edge of the island's monitor,
    /// screen pixels) or off: the full-screen watcher reports what reaches it.
    pub fn watch_top(&self, band: Option<RECT>) {
        if let Some(f) = &self.fullscreen {
            f.watch_top(band);
        }
    }

    /// Explorer restarted: AppBar registrations are lost and must be redone.
    pub fn on_taskbar_created(&self) {
        if let Some(f) = &self.fullscreen {
            f.on_taskbar_created();
        }
    }

    pub fn media_cmd(&self, c: MediaCmd) {
        if let Some(m) = &self.media {
            m.send(c);
        }
    }

    pub fn request_lyrics(&self, s: &MediaSession) {
        if let Some(l) = &self.lyrics {
            l.request(s);
        }
    }

    /// Handle the AppBar callback message; `Some(true)` when a full-screen
    /// app covers the island's monitor.
    pub fn on_appbar_message(&self, wp: WPARAM, lp: LPARAM, monitor: isize) -> Option<bool> {
        self.fullscreen.as_ref().and_then(|f| f.on_appbar_message(wp, lp, monitor))
    }
}

/// Windows apps are set to light mode (Settings › Personalization › Colors).
pub fn apps_use_light_theme() -> bool {
    use windows::UI::ViewManagement::{UIColorType, UISettings};
    // The system background is white in light mode and black in dark mode.
    UISettings::new().and_then(|s| s.GetColorValue(UIColorType::Background)).map(|c| c.R as u32 + c.G as u32 + c.B as u32 > 382).unwrap_or(true)
}

/// The Windows accent colour as it is: the Glass theme's text and fills sit
/// on light and dark glass alike, so it isn't lightened for black.
pub fn system_accent_base() -> Option<Color> {
    use windows::UI::ViewManagement::{UIColorType, UISettings};
    let s = UISettings::new().ok()?;
    let c = s.GetColorValue(UIColorType::Accent).ok()?;
    Some(Color::rgba(c.R as f32 / 255.0, c.G as f32 / 255.0, c.B as f32 / 255.0, 1.0))
}

/// The Windows accent colour, lightened for use on the black island.
pub fn system_accent() -> Option<Color> {
    use windows::UI::ViewManagement::{UIColorType, UISettings};
    let s = UISettings::new().ok()?;
    let c = s.GetColorValue(UIColorType::AccentLight2).ok()?;
    Some(Color::rgba(c.R as f32 / 255.0, c.G as f32 / 255.0, c.B as f32 / 255.0, 1.0).for_island_accent())
}
