//! Side effects requested by pages; executed by the app after each frame.

use crate::pages::PageKind;

#[derive(Clone, Debug, PartialEq)]
pub enum MediaCmd {
    PlayPause,
    Next,
    Prev,
    /// Seek to absolute position in seconds.
    Seek(f64),
    /// The pointer reached the progress bar or the back button: get ready
    /// (Apple Music's own controls are looked up before the press, not during it).
    Prepare,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sound {
    /// Gentle chime (timer finished, reminder).
    Chime,
    /// Short tick (button feedback) — currently unused.
    Tick,
}

/// Phone drop (the Shelf's phone panel).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhoneCmd {
    /// Start it (or try again after an error).
    Open,
    /// Stop it; files still coming in are dropped.
    Close,
    /// Answer the phone's question.
    Answer { id: u64, accept: bool },
}

/// Requests for Notifications (`services::notifications`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotifyCmd {
    /// Ask Windows to let the island read notifications (its question).
    Ask,
    /// Look again at what Windows allows: the page opened, and the person
    /// may have changed it in Windows' settings meanwhile.
    Check,
    /// Open Windows' settings for which apps may read notifications.
    OpenWindowsSettings,
}

/// The IELTS page's buttons; `poni_core::ielts::Coach` decides what each
/// does in the moment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IeltsCmd {
    /// Mulai (and Lagi on the result): a session at the picked level.
    Start,
    /// The mic button: answer now, or stop the answer.
    Mic,
    /// Lewati: the next question, this one unscored.
    Skip,
    /// Udahan or Selesai: end the session.
    Quit,
    /// Coba lagi.
    Retry,
}

/// The Notes page's buttons (the Meeting notes add-on).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotesCmd {
    /// Take notes: Oyen starts listening.
    Start,
    /// Stop: the meeting is over, its notes get written.
    Stop,
    /// Try again: hand the kept recording over again.
    Retry,
    /// Throw the kept recording away.
    Discard,
    /// Open the notes' file, or the notes folder.
    Open,
    Folder,
    /// Copy the notes (without the transcript) as Markdown.
    Copy,
    /// Back to Take notes.
    Done,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Media(MediaCmd),
    /// Open a file/folder with its default handler.
    OpenPath(String),
    /// Show a file in Explorer.
    RevealPath(String),
    /// Begin an OLE drag of these paths out of the island.
    DragOut(Vec<String>),
    /// Request a thumbnail for a shelf item (async; arrives as `AppEvent::ShelfThumb`).
    LoadThumb(String),
    /// Give the island keyboard focus for a text field.
    FocusText,
    /// The Script page's Paste: the clipboard's text, line breaks kept,
    /// becomes the script.
    ScriptPaste,
    /// The Script page's Record: Oyenza Studio Recorder records the screen
    /// the island is on (`services::recorder`).
    RecordScreen,
    /// The Record page's Record: a recording of `settings.record`'s target
    /// with its camera, microphone and system sound (`services::recorder`).
    Record,
    /// Pause, resume or stop the recording under way (the Record page).
    RecordPause,
    RecordResume,
    RecordStop,
    /// Drop keyboard focus.
    BlurText,
    /// Persist `AppState::store` soon.
    SaveStore,
    /// Persist `AppState::settings` now (after a page toggled a setting).
    SaveSettings,
    /// Open the Settings window, or bring it forward (the gear, Windows).
    OpenSettings,
    SetPage(PageKind),
    /// Keep the island open even when the cursor leaves.
    SetPinned(bool),
    Collapse,
    Play(Sound),
    /// Open a URL in the default browser.
    OpenUrl(String),
    /// Start/stop the 1 Hz system-stats sampler.
    WantStats(bool),
    /// Start with Windows on or off (the Settings page's switch).
    SetAutostart(bool),
    /// Open the System page on this module (a Today ring was clicked).
    OpenSystem(poni_core::sysmon::Module),
    /// Request an app's icon for the System page (arrives in `AppState::app_icons`).
    LoadAppIcon(String),
    /// Show over games: find out whether Windows lets Poni read frames
    /// (the answer lands in `AppState::game.access`).
    CheckFps,
    /// Show over games: ask Windows once for the right to read frames.
    AllowFps,
    Phone(PhoneCmd),
    /// Notifications.
    Notify(NotifyCmd),
}
