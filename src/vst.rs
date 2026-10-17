//! Small VST3 effect host for the offline mastering path.
//!
//! VST bundles are discovered without loading their native code. Loading, GUI
//! creation and project-state capture happen only after an explicit user action.
use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Vst3Slot {
    #[serde(default)] pub path: String,
    #[serde(default)] pub enabled: bool,
    #[serde(default)] pub bypass: bool,
    #[serde(default)] pub state: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Vst3PluginFile {
    pub name: String,
    pub path: String,
}

fn standard_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    #[cfg(target_os = "windows")]
    {
        paths.push(PathBuf::from(r"C:\Program Files\Common Files\VST3"));
        paths.push(PathBuf::from(r"C:\Program Files (x86)\Common Files\VST3"));
    }
    #[cfg(target_os = "macos")]
    {
        paths.push(PathBuf::from("/Library/Audio/Plug-Ins/VST3"));
        if let Some(home) = dirs::home_dir() { paths.push(home.join("Library/Audio/Plug-Ins/VST3")); }
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(home) = dirs::home_dir() { paths.push(home.join(".vst3")); }
        paths.extend([PathBuf::from("/usr/lib/vst3"), PathBuf::from("/usr/local/lib/vst3"), PathBuf::from("/usr/lib/x86_64-linux-gnu/vst3")]);
    }
    paths
}

fn walk_plugins(dir: &Path, depth: usize, found: &mut Vec<Vst3PluginFile>) {
    if depth > 6 { return; }
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(kind) = entry.file_type() else { continue };
        if kind.is_dir() {
            if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("vst3")) {
                let canonical = path.canonicalize().unwrap_or(path);
                let name = canonical.file_stem().or_else(|| canonical.file_name()).unwrap_or_default().to_string_lossy().into_owned();
                found.push(Vst3PluginFile { name, path: canonical.to_string_lossy().into_owned() });
            } else {
                walk_plugins(&path, depth + 1, found);
            }
        } else if kind.is_file() && path.extension().is_some_and(|e| e.eq_ignore_ascii_case("vst3")) {
            let canonical = path.canonicalize().unwrap_or(path);
            let name = canonical.file_stem().unwrap_or_default().to_string_lossy().into_owned();
            found.push(Vst3PluginFile { name, path: canonical.to_string_lossy().into_owned() });
        }
    }
}

pub fn scan() -> Vec<Vst3PluginFile> {
    let mut found = Vec::new();
    for path in standard_paths() { walk_plugins(&path, 0, &mut found); }
    found.sort_by(|a,b| a.name.to_lowercase().cmp(&b.name.to_lowercase()).then_with(|| a.path.cmp(&b.path)));
    found.dedup_by(|a,b| a.path == b.path);
    found
}

#[cfg(not(target_os = "freebsd"))]
#[tauri::command]
pub fn scan_plugins() -> Vec<Vst3PluginFile> { scan() }

fn approved_plugin_path(raw: &str) -> Result<PathBuf, String> {
    let path = Path::new(raw).canonicalize().map_err(|e| format!("VST3-sökvägen finns inte: {e}"))?;
    if !path.extension().is_some_and(|e| e.eq_ignore_ascii_case("vst3")) {
        return Err("Välj en .vst3-plugin från den skannade listan".into());
    }
    if !scan().iter().any(|plugin| Path::new(&plugin.path) == path) {
        return Err("Pluginet finns inte i någon skannad standardmapp".into());
    }
    Ok(path)
}

#[cfg(not(target_os = "freebsd"))]
fn decode_state(state: Option<&str>) -> Result<Option<Vec<u8>>, String> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let Some(encoded) = state.filter(|s| !s.is_empty()) else { return Ok(None) };
    if encoded.len() > 28_000_000 { return Err("VST3-state är för stor".into()); }
    STANDARD.decode(encoded).map(Some).map_err(|_| "Ogiltig base64-data för VST3-state".into())
}

/// Processes an interleaved stereo buffer in offline blocks immediately before
/// the final limiter/dither. Bypass preserves the input bit-for-bit.
#[cfg(not(target_os = "freebsd"))]
pub fn process_offline(audio: &mut [f32], sample_rate: u32, slot: &Vst3Slot) -> Result<(), String> {
    use vst3_host::{audio::AudioBuffers, ProcessMode, Vst3Host};
    if !slot.enabled || slot.bypass { return Ok(()); }
    if audio.len() % 2 != 0 || audio.iter().any(|v| !v.is_finite()) { return Err("VST3 kräver giltigt interleaved stereo-ljud".into()); }
    let path = approved_plugin_path(&slot.path)?;
    let mut host = Vst3Host::builder().sample_rate(sample_rate as f64).block_size(512).build().map_err(|e| e.to_string())?;
    let mut plugin = host.load_plugin(&path).map_err(|e| format!("Kunde inte ladda VST3: {e}"))?;
    if let Some(state) = decode_state(slot.state.as_deref())? {
        plugin.load_state(&state).map_err(|e| format!("Kunde inte återställa VST3-inställningarna: {e}"))?;
    }
    plugin.set_process_mode(ProcessMode::Offline).map_err(|e| e.to_string())?;
    plugin.start_processing().map_err(|e| e.to_string())?;
    let frames = audio.len()/2;
    let mut result = Vec::with_capacity(audio.len());
    let mut offset = 0;
    let mut failure = None;
    while offset < frames {
        let count = (frames-offset).min(512);
        let mut buffers = AudioBuffers::new(2, 2, count, sample_rate as f64);
        for frame in 0..count {
            buffers.inputs[0][frame] = audio[(offset+frame)*2];
            buffers.inputs[1][frame] = audio[(offset+frame)*2+1];
        }
        if let Err(e) = plugin.process_audio(&mut buffers) { failure = Some(e.to_string()); break; }
        if buffers.outputs.len() < 2 || buffers.outputs[0].len() < count || buffers.outputs[1].len() < count {
            failure = Some("VST3-pluginet returnerade inte två ljudkanaler".into()); break;
        }
        for frame in 0..count { result.push(buffers.outputs[0][frame]); result.push(buffers.outputs[1][frame]); }
        offset += count;
    }
    let stop = plugin.stop_processing().map_err(|e| e.to_string());
    if let Some(error) = failure { return Err(format!("VST3-processning misslyckades: {error}")); }
    stop?;
    if result.iter().any(|v| !v.is_finite()) { return Err("VST3-pluginet returnerade ogiltiga ljudvärden".into()); }
    audio.copy_from_slice(&result);
    Ok(())
}

#[cfg(target_os = "freebsd")]
pub fn process_offline(_: &mut [f32], _: u32, _: &Vst3Slot) -> Result<(), String> {
    Err("VST3-värden är inte tillgänglig på FreeBSD".into())
}

#[cfg(not(target_os = "freebsd"))]
thread_local! {
    static OPEN_PLUGINS: std::cell::RefCell<std::collections::HashMap<String,
        (std::sync::Arc<std::sync::Mutex<vst3_host::Plugin>>, vst3_host::PluginWindow)>> = std::cell::RefCell::new(std::collections::HashMap::new());
}

#[cfg(not(target_os = "freebsd"))]
fn start_editor_pump(app: tauri::AppHandle) {
    static STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if STARTED.swap(true, std::sync::atomic::Ordering::AcqRel) { return; }
    let pending = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(16));
        if pending.compare_exchange(false, true, std::sync::atomic::Ordering::AcqRel, std::sync::atomic::Ordering::Acquire).is_err() { continue; }
        let pending_callback = pending.clone();
        let app = app.clone();
        if app.run_on_main_thread(move || {
            OPEN_PLUGINS.with(|open| open.borrow_mut().retain(|_, (_, window)| {
                let _ = window.service_platform_events();
                if window.closed_by_user() { window.close(); }
                true
            }));
            pending_callback.store(false, std::sync::atomic::Ordering::Release);
        }).is_err() { break; }
    });
}

#[cfg(not(target_os = "freebsd"))]
#[tauri::command]
pub fn open_plugin(app: tauri::AppHandle, path: String, sample_rate: u32, state: Option<String>, profile_id: String) -> Result<(), String> {
    use std::sync::{Arc, Mutex, mpsc};
    use tauri::Manager;
    use vst3_host::Vst3Host;
    let path = approved_plugin_path(&path)?;
    if !["Spotify", "Vinyl", "CD", "Kassett"].contains(&profile_id.as_str()) { return Err("Okänd masteringprofil".into()); }
    let key = format!("{profile_id}\0{}", path.to_string_lossy());
    let decoded_state = decode_state(state.as_deref())?;
    let (tx, rx) = mpsc::channel();
    let pump_app = app.clone();
    app.run_on_main_thread(move || {
        let result = (|| {
            let existing = OPEN_PLUGINS.with(|open| {
                let mut open = open.borrow_mut();
                if let Some((_, window)) = open.get_mut(&key) {
                    if !window.is_open() { window.open().map_err(|e|e.to_string())?; }
                    Ok(true)
                } else { Ok(false) }
            })?;
            if existing { return Ok(()); }
            let mut host = Vst3Host::builder().sample_rate(sample_rate.clamp(8000,192000) as f64).block_size(512).build().map_err(|e|e.to_string())?;
            let mut plugin = host.load_plugin(&key).map_err(|e|format!("Kunde inte ladda VST3: {e}"))?;
            if let Some(bytes) = decoded_state { plugin.load_state(&bytes).map_err(|e|format!("Kunde inte återställa VST3-inställningarna: {e}"))?; }
            if !plugin.has_editor() { return Err("Pluginet har inget eget GUI-fönster".into()); }
            let plugin = Arc::new(Mutex::new(plugin));
            let mut window = vst3_host::PluginWindow::new(plugin.clone());
            window.open().map_err(|e|e.to_string())?;
            OPEN_PLUGINS.with(|open| { open.borrow_mut().insert(key, (plugin, window)); });
            start_editor_pump(pump_app);
            Ok(())
        })();
        let _ = tx.send(result);
    }).map_err(|e|e.to_string())?;
    rx.recv().map_err(|e|e.to_string())?
}

#[cfg(not(target_os = "freebsd"))]
#[tauri::command]
pub fn save_plugin_state(app: tauri::AppHandle, path: String, profile_id: String) -> Result<Option<String>, String> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use std::sync::mpsc;
    use tauri::Manager;
    let path = approved_plugin_path(&path)?;
    if !["Spotify", "Vinyl", "CD", "Kassett"].contains(&profile_id.as_str()) { return Err("Okänd masteringprofil".into()); }
    let key = format!("{profile_id}\0{}", path.to_string_lossy());
    let (tx, rx) = mpsc::channel();
    app.run_on_main_thread(move || {
        let state = OPEN_PLUGINS.with(|open| {
            let open = open.borrow();
            open.get(&key).map(|(plugin, _)| plugin.lock().map_err(|_| "VST3-pluginets lås är skadad".to_string())?.save_state().map_err(|e|e.to_string()))
        });
        let result = match state { Some(Ok(bytes)) => Ok(Some(STANDARD.encode(bytes))), Some(Err(e)) => Err(e), None => Ok(None) };
        let _ = tx.send(result);
    }).map_err(|e|e.to_string())?;
    rx.recv().map_err(|e|e.to_string())?
}

#[cfg(target_os = "freebsd")]
pub fn scan_plugins() -> Vec<Vst3PluginFile> { Vec::new() }
