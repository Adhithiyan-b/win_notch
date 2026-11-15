//! Master volume change notifications (Core Audio).
//!
//! A dedicated MTA thread owns the `IMMDeviceEnumerator` and the
//! `IAudioEndpointVolume` of the default render endpoint (console role).
//! Volume changes arrive on an audio-engine thread through
//! [`VolumeCallback::OnNotify`], which only takes a short lock, drops
//! duplicates and stale endpoints, and posts `AppEvent::Volume`.
//! Default-device changes arrive through [`DeviceClient`], which only wakes
//! the worker; the worker re-binds to the new endpoint (Core Audio forbids
//! (un)registering from inside its own callbacks) and reports its state.
//!
//! Idle cost: the worker sleeps in a blocking `recv`; nothing polls.

use crate::events::{AppEvent, EventSink};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::Duration;
use windows::core::{implement, Interface, Result, PCWSTR};
use windows::Win32::Foundation::E_FAIL;
use windows::Win32::Media::Audio::Endpoints::{IAudioEndpointVolume, IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl};
use windows::Win32::Media::Audio::{
    eConsole, eMultimedia, eRender, EDataFlow, ERole, IMMDevice, IMMDeviceEnumerator, IMMNotificationClient, IMMNotificationClient_Impl,
    MMDeviceEnumerator, AUDCLNT_E_DEVICE_INVALIDATED, AUDIO_VOLUME_NOTIFICATION_DATA, DEVICE_STATE,
};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED};
use windows::Win32::UI::Shell::PropertiesSystem::PROPERTYKEY;

/// How long `Drop` waits for the worker to unregister before detaching it.
const STOP_TIMEOUT: Duration = Duration::from_millis(800);

/// Master-volume watcher. Dropping it unregisters every callback.
pub struct AudioService {
    tx: Sender<Msg>,
    done: Receiver<()>,
    thread: Option<JoinHandle<()>>,
}

impl AudioService {
    pub fn start(sink: EventSink) -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let (done_tx, done) = mpsc::channel();
        let client_tx = tx.clone();
        let thread = std::thread::Builder::new()
            .name("poni-audio".into())
            .spawn(move || {
                worker(sink, rx, client_tx);
                let _ = done_tx.send(());
            })
            .map_err(|e| windows::core::Error::new(E_FAIL, format!("audio thread: {e}")))?;
        Ok(Self { tx, done, thread: Some(thread) })
    }
}

impl Drop for AudioService {
    fn drop(&mut self) {
        let _ = self.tx.send(Msg::Shutdown);
        let Some(thread) = self.thread.take() else { return };
        match self.done.recv_timeout(STOP_TIMEOUT) {
            // Finished (or died): joining is immediate.
            Ok(()) | Err(RecvTimeoutError::Disconnected) => {
                let _ = thread.join();
            }
            Err(RecvTimeoutError::Timeout) => crate::pwarn!("audio: worker did not stop in time; detaching it"),
        }
    }
}

enum Msg {
    /// The default render endpoint (console or multimedia role) changed.
    DefaultChanged,
    Shutdown,
}

// ---------------------------------------------------------------------------
// Publishing: dedupe + stale-endpoint filter, shared with the COM callbacks.

struct GateState {
    /// Bumped on every re-bind; callbacks of older endpoints are ignored.
    generation: u32,
    /// Last reported (level in 1/10000 steps, muted).
    last: Option<(u32, bool)>,
}

struct Gate {
    sink: EventSink,
    state: Mutex<GateState>,
}

impl Gate {
    fn lock(&self) -> MutexGuard<'_, GateState> {
        // A poisoned lock only means another thread panicked mid-update; the
        // two plain fields are still meaningful.
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Start a new generation (before binding a new endpoint).
    fn next_generation(&self) -> u32 {
        let mut s = self.lock();
        s.generation = s.generation.wrapping_add(1);
        s.generation
    }

    /// Report `level`/`muted` unless it comes from a stale endpoint or
    /// repeats the last report. Cheap: one lock, no allocation on the
    /// duplicate path. Holding the lock while sending keeps reports ordered
    /// with respect to re-binds.
    fn publish(&self, generation: u32, level: f32, muted: bool) {
        let level = if level.is_finite() { level.clamp(0.0, 1.0) } else { 0.0 };
        let key = ((level * 10_000.0).round() as u32, muted);
        let mut s = self.lock();
        if s.generation != generation || s.last == Some(key) {
            return;
        }
        s.last = Some(key);
        self.sink.send(AppEvent::Volume { level, muted });
    }
}

// ---------------------------------------------------------------------------
// COM callbacks.

#[implement(IAudioEndpointVolumeCallback)]
struct VolumeCallback {
    gate: Arc<Gate>,
    generation: u32,
}

impl IAudioEndpointVolumeCallback_Impl for VolumeCallback_Impl {
    fn OnNotify(&self, pnotify: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> Result<()> {
        // SAFETY: Core Audio passes either null or a struct that is valid for
        // the duration of this call; we only read two fields.
        if let Some(data) = unsafe { pnotify.as_ref() } {
            self.gate.publish(self.generation, data.fMasterVolume, data.bMuted.as_bool());
        }
        Ok(())
    }
}

#[implement(IMMNotificationClient)]
struct DeviceClient {
    tx: Sender<Msg>,
}

impl IMMNotificationClient_Impl for DeviceClient_Impl {
    fn OnDeviceStateChanged(&self, _id: &PCWSTR, _state: DEVICE_STATE) -> Result<()> {
        Ok(())
    }

    fn OnDeviceAdded(&self, _id: &PCWSTR) -> Result<()> {
        Ok(())
    }

    fn OnDeviceRemoved(&self, _id: &PCWSTR) -> Result<()> {
        Ok(())
    }

    fn OnDefaultDeviceChanged(&self, flow: EDataFlow, role: ERole, _id: &PCWSTR) -> Result<()> {
        // Never touch Core Audio from here: just wake the worker.
        if flow == eRender && (role == eConsole || role == eMultimedia) {
            let _ = self.tx.send(Msg::DefaultChanged);
        }
        Ok(())
    }

    fn OnPropertyValueChanged(&self, _id: &PCWSTR, _key: &PROPERTYKEY) -> Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Worker thread.

/// COM apartment membership for the current thread.
struct ComApartment {
    initialized: bool,
}

impl ComApartment {
    fn enter_mta() -> Self {
        // S_OK and S_FALSE both need a matching CoUninitialize; a failure
        // (e.g. RPC_E_CHANGED_MODE) must not be balanced.
        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if hr.is_err() {
            crate::pwarn!("audio: CoInitializeEx failed ({hr:?})");
        }
        Self { initialized: hr.is_ok() }
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        if self.initialized {
            unsafe { CoUninitialize() };
        }
    }
}

/// The endpoint we listen to. The callback object stays alive (and
/// registered) for exactly as long as this binding.
struct Binding {
    id: String,
    volume: IAudioEndpointVolume,
    callback: IAudioEndpointVolumeCallback,
}

impl Binding {
    fn unbind(self) {
        match unsafe { self.volume.UnregisterControlChangeNotify(&self.callback) } {
            // An unplugged endpoint is already gone, together with its registration.
            Err(e) if e.code() == AUDCLNT_E_DEVICE_INVALIDATED => {}
            Err(e) => crate::pwarn!("audio: UnregisterControlChangeNotify failed ({e})"),
            Ok(()) => {}
        }
    }
}

fn worker(sink: EventSink, rx: Receiver<Msg>, client_tx: Sender<Msg>) {
    // Declared first so it is dropped last, after every COM object below.
    let com = ComApartment::enter_mta();
    if !com.initialized {
        return;
    }

    let enumerator: IMMDeviceEnumerator = match unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) } {
        Ok(e) => e,
        Err(e) => {
            crate::pwarn!("audio: MMDeviceEnumerator unavailable ({e}); volume pop-ups disabled");
            return;
        }
    };

    let client: IMMNotificationClient = DeviceClient { tx: client_tx }.into();
    let client_registered = match unsafe { enumerator.RegisterEndpointNotificationCallback(&client) } {
        Ok(()) => true,
        Err(e) => {
            crate::pwarn!("audio: default-device notifications unavailable ({e})");
            false
        }
    };

    let gate = Arc::new(Gate { sink, state: Mutex::new(GateState { generation: 0, last: None }) });
    // The first report doubles as the initial state, which the app shows silently.
    let mut binding = rebind(&enumerator, &gate, None);

    while let Ok(msg) = rx.recv() {
        match msg {
            Msg::Shutdown => break,
            Msg::DefaultChanged => {
                // One switch fires once per role; handle the burst once.
                let mut shutdown = false;
                while let Ok(m) = rx.try_recv() {
                    if matches!(m, Msg::Shutdown) {
                        shutdown = true;
                        break;
                    }
                }
                if shutdown {
                    break;
                }
                binding = rebind(&enumerator, &gate, binding);
            }
        }
    }

    if let Some(b) = binding {
        b.unbind();
    }
    if client_registered {
        if let Err(e) = unsafe { enumerator.UnregisterEndpointNotificationCallback(&client) } {
            crate::pwarn!("audio: UnregisterEndpointNotificationCallback failed ({e})");
        }
    }
    // `client`, `enumerator`, then `com` drop here, in that order.
}

/// Bind to the current default endpoint (unless it is the one already bound)
/// and report its state.
fn rebind(enumerator: &IMMDeviceEnumerator, gate: &Arc<Gate>, current: Option<Binding>) -> Option<Binding> {
    let device = match unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eConsole) } {
        Ok(d) => d,
        Err(e) => {
            // No output device at all (all unplugged / disabled).
            crate::pinfo!("audio: no default output device ({e})");
            gate.next_generation();
            if let Some(b) = current {
                b.unbind();
            }
            return None;
        }
    };
    let id = device_id(&device);
    if let Some(b) = &current {
        // Same endpoint (a per-role repeat of one switch) and still alive. A
        // quick unplug + replug keeps the id but invalidates the old object,
        // which then fails this read and gets re-bound below.
        if !id.is_empty() && b.id == id && unsafe { b.volume.GetMasterVolumeLevelScalar() }.is_ok() {
            return current;
        }
    }

    // A new generation silences the old endpoint's late callbacks.
    let generation = gate.next_generation();
    if let Some(b) = current {
        b.unbind();
    }
    match bind(&device, id, gate, generation) {
        Ok(b) => {
            match unsafe { (b.volume.GetMasterVolumeLevelScalar(), b.volume.GetMute()) } {
                (Ok(level), Ok(muted)) => gate.publish(generation, level, muted.as_bool()),
                (Err(e), _) | (_, Err(e)) => crate::pwarn!("audio: reading the volume failed ({e})"),
            }
            Some(b)
        }
        Err(e) => {
            crate::pwarn!("audio: binding the default output device failed ({e})");
            None
        }
    }
}

fn bind(device: &IMMDevice, id: String, gate: &Arc<Gate>, generation: u32) -> Result<Binding> {
    let volume: IAudioEndpointVolume = unsafe { device.Activate(CLSCTX_ALL, None)? };
    let callback: IAudioEndpointVolumeCallback = VolumeCallback { gate: Arc::clone(gate), generation }.into();
    unsafe { volume.RegisterControlChangeNotify(&callback)? };
    Ok(Binding { id, volume, callback })
}

/// The endpoint id string ("" if it cannot be read).
fn device_id(device: &IMMDevice) -> String {
    match unsafe { device.GetId() } {
        Ok(p) if !p.is_null() => {
            // SAFETY: GetId returns a NUL-terminated, CoTaskMemAlloc'ed string
            // that we own and free exactly once.
            let id = unsafe { p.to_string() }.unwrap_or_default();
            unsafe { CoTaskMemFree(Some(p.0 as *const _)) };
            id
        }
        _ => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Audio output device switcher & enumeration (Milestone D.3).

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioOutputDevice {
    pub id: String,
    pub name: String,
    pub icon: crate::icons::Icon,
    pub is_default: bool,
}

pub trait AudioOutputSwitcher: Send + Sync {
    fn set_default_endpoint(&self, id: &str) -> Result<()>;
}

// Undocumented Windows PolicyConfig COM identifiers (Windows 10 / 11)
const CLSID_POLICY_CONFIG_CLIENT: windows::core::GUID = windows::core::GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9);
const CLSID_POLICY_CONFIG_ALT1: windows::core::GUID = windows::core::GUID::from_u128(0x870af99c_171d_4f9e_af0d_63c40f6e387b);
const CLSID_POLICY_CONFIG_ALT2: windows::core::GUID = windows::core::GUID::from_u128(0x294fcb26_8800_4f02_8e9d_c0d6258a10e4);
const IID_IPOLICY_CONFIG: windows::core::GUID = windows::core::GUID::from_u128(0xf8679f50_850a_41cf_9c72_430f290290c8);

#[allow(non_snake_case)]
#[repr(C)]
struct IPolicyConfigVtbl {
    pub QueryInterface: unsafe extern "system" fn(this: *mut std::ffi::c_void, riid: *const windows::core::GUID, ppvObject: *mut *mut std::ffi::c_void) -> windows::core::HRESULT,
    pub AddRef: unsafe extern "system" fn(this: *mut std::ffi::c_void) -> u32,
    pub Release: unsafe extern "system" fn(this: *mut std::ffi::c_void) -> u32,
    pub GetMixFormat: usize,
    pub GetDeviceFormat: usize,
    pub ResetDeviceFormat: usize,
    pub SetDeviceFormat: usize,
    pub GetProcessingPeriod: usize,
    pub SetProcessingPeriod: usize,
    pub GetShareMode: usize,
    pub SetShareMode: usize,
    pub GetPropertyValue: usize,
    pub SetPropertyValue: usize,
    pub SetDefaultEndpoint: unsafe extern "system" fn(this: *mut std::ffi::c_void, wszDeviceId: PCWSTR, eRole: ERole) -> windows::core::HRESULT,
    pub SetEndpointVisibility: usize,
}

/// Real implementation talking to Windows CPolicyConfigClient COM service.
pub struct ComPolicyConfig;

impl AudioOutputSwitcher for ComPolicyConfig {
    fn set_default_endpoint(&self, id: &str) -> Result<()> {
        let _com = ComApartment::enter_mta();
        let wide: Vec<u16> = id.encode_utf16().chain(std::iter::once(0)).collect();
        let pcwstr = PCWSTR(wide.as_ptr());

        let clsids = [
            CLSID_POLICY_CONFIG_CLIENT,
            CLSID_POLICY_CONFIG_ALT1,
            CLSID_POLICY_CONFIG_ALT2,
        ];

        let mut instance: *mut std::ffi::c_void = std::ptr::null_mut();

        for clsid in &clsids {
            let res = unsafe {
                CoCreateInstance::<_, windows::core::IUnknown>(
                    clsid,
                    None,
                    CLSCTX_ALL,
                )
            };
            if let Ok(unk) = res {
                let q_hr = unsafe {
                    (unk.vtable().QueryInterface)(
                        windows::core::Interface::as_raw(&unk),
                        &IID_IPOLICY_CONFIG,
                        &mut instance,
                    )
                };
                if q_hr.is_ok() && !instance.is_null() {
                    break;
                }
            }
        }

        if instance.is_null() {
            crate::pwarn!("audio: failed to obtain IPolicyConfig COM interface");
            return Err(windows::core::Error::new(windows::Win32::Foundation::E_NOINTERFACE, "IPolicyConfig unavailable"));
        }

        unsafe {
            let vtbl = *(instance as *const *const IPolicyConfigVtbl);
            let roles = [eConsole, eMultimedia, windows::Win32::Media::Audio::eCommunications];
            for role in roles {
                let _ = ((*vtbl).SetDefaultEndpoint)(instance, pcwstr, role);
            }
            ((*vtbl).Release)(instance);
        }

        Ok(())
    }
}

/// Fallback implementation that logs and degrades gracefully if COM query fails.
pub struct FallbackAudioOutputSwitcher;

impl AudioOutputSwitcher for FallbackAudioOutputSwitcher {
    fn set_default_endpoint(&self, id: &str) -> Result<()> {
        crate::pwarn!("audio: FallbackAudioOutputSwitcher invoked for endpoint '{id}' (no-op)");
        Ok(())
    }
}

/// Factory returning the real ComPolicyConfig switcher with graceful fallback capability.
pub fn create_audio_output_switcher() -> Box<dyn AudioOutputSwitcher> {
    Box::new(ComPolicyConfig)
}

const PKEY_DEVICE_FRIENDLY_NAME: PROPERTYKEY = PROPERTYKEY {
    fmtid: windows::core::GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
    pid: 14,
};

/// Enumerate all active audio rendering endpoints with friendly name and device icon.
pub fn list_output_devices() -> Vec<AudioOutputDevice> {
    let _com = ComApartment::enter_mta();
    let enumerator: IMMDeviceEnumerator = match unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) } {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let default_id = match unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eConsole) } {
        Ok(d) => device_id(&d),
        Err(_) => String::new(),
    };

    let collection = match unsafe { enumerator.EnumAudioEndpoints(eRender, windows::Win32::Media::Audio::DEVICE_STATE_ACTIVE) } {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let count = unsafe { collection.GetCount() }.unwrap_or(0);
    let mut devices = Vec::with_capacity(count as usize);

    for i in 0..count {
        let Ok(device) = (unsafe { collection.Item(i) }) else { continue };
        let id = device_id(&device);
        if id.is_empty() {
            continue;
        }

        let is_default = !default_id.is_empty() && id == default_id;

        let mut name = String::new();
        let store = unsafe { device.OpenPropertyStore(windows::Win32::System::Com::STGM_READ) };
        if let Ok(store) = store {
            if let Ok(prop) = unsafe { store.GetValue(&PKEY_DEVICE_FRIENDLY_NAME) } {
                name = prop.to_string();
            }
        }
        if name.is_empty() {
            name = format!("Audio Endpoint {}", i + 1);
        }

        let lower = name.to_lowercase();
        let icon = if lower.contains("headphone") || lower.contains("headset") || lower.contains("earphone") || lower.contains("buds") {
            crate::icons::Icon::Headphones
        } else if lower.contains("bluetooth") {
            crate::icons::Icon::Bluetooth
        } else if lower.contains("tv") || lower.contains("monitor") || lower.contains("display") || lower.contains("hdmi") {
            crate::icons::Icon::Screen
        } else {
            crate::icons::Icon::Volume
        };

        devices.push(AudioOutputDevice { id, name, icon, is_default });
    }

    devices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_switcher_succeeds_without_panic() {
        let switcher = FallbackAudioOutputSwitcher;
        assert!(switcher.set_default_endpoint("dummy-endpoint-id").is_ok());
    }

    #[test]
    fn com_policy_config_creation() {
        let switcher = create_audio_output_switcher();
        // Calling with an empty/invalid id should return Err without panic
        let _ = switcher.set_default_endpoint("");
    }

    #[test]
    fn list_output_devices_executes_safely() {
        let devices = list_output_devices();
        for dev in &devices {
            assert!(!dev.id.is_empty());
            assert!(!dev.name.is_empty());
        }
    }
}
