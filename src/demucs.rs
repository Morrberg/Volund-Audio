//! Optional external Demucs-RS runner. Model weights are fetched on first use.
use crate::{ext, fname, is_audio, sess};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderValue},
    response::Response,
    Json,
};
use bytes::Bytes;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    convert::Infallible,
    fs::{self, File},
    io::{Read, Write},
    path::{Path as FsPath, PathBuf},
    process::Command,
    sync::mpsc::Sender,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(Deserialize)]
pub struct Request {
    pub stem: String,
}

struct TempOutput(PathBuf);
impl Drop for TempOutput {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn executable() -> PathBuf {
    if let Some(path) = std::env::var_os("VOLUND_DEMUCS_PATH") {
        return PathBuf::from(path);
    }
    let name = if cfg!(windows) { "demucs.exe" } else { "demucs" };
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let sibling = parent.join(name);
            if sibling.is_file() {
                return sibling;
            }
        }
    }
    PathBuf::from(name)
}

type Events = mpsc::UnboundedSender<Result<Bytes, Infallible>>;
fn emit(events: &Events, value: Value) {
    if let Ok(data) = serde_json::to_string(&value) {
        let _ = events.send(Ok(Bytes::from(format!("data: {data}\n\n"))));
    }
}

fn prepare_model(events: &Events) -> Result<(), String> {
    let cache = dirs::cache_dir()
        .ok_or_else(|| "Kunde inte hitta systemets cachemapp.".to_string())?
        .join("demucs-rs");
    let model = cache.join("htdemucs.safetensors");
    if model.is_file() {
        emit(events, json!({"type":"phase","phase":"model-ready"}));
        return Ok(());
    }
    fs::create_dir_all(&cache).map_err(|e| format!("Kunde inte skapa modellcachen: {e}"))?;
    let temp = cache.join(format!("htdemucs-{}.download", std::process::id()));
    let url = "https://huggingface.co/set-soft/audio_separation/resolve/main/Demucs/htdemucs.safetensors";
    let response = match ureq::get(url).call() {
        Ok(response) => response,
        Err(ureq::Error::Status(code, _)) => return Err(format!("Modellservern svarade med HTTP {code}.")),
        Err(ureq::Error::Transport(error)) => return Err(format!("Kunde inte hämta modellvikterna: {error}")),
    };
    let total = response
        .header("Content-Length")
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(84_000_000);
    let mut reader = response.into_reader();
    let mut file = File::create(&temp).map_err(|e| format!("Kunde inte skapa nedladdningsfilen: {e}"))?;
    let mut buffer = [0u8; 64 * 1024];
    let mut current = 0u64;
    emit(events, json!({"type":"progress","phase":"download","current":0,"total":total}));
    loop {
        let count = reader.read(&mut buffer).map_err(|e| format!("Nedladdningen avbröts: {e}"))?;
        if count == 0 { break; }
        file.write_all(&buffer[..count]).map_err(|e| format!("Kunde inte spara modellvikterna: {e}"))?;
        current += count as u64;
        emit(events, json!({"type":"progress","phase":"download","current":current,"total":total}));
    }
    file.sync_all().map_err(|e| format!("Kunde inte slutföra modellfilen: {e}"))?;
    if current < total {
        let _ = fs::remove_file(&temp);
        return Err(format!("Nedladdningen blev ofullständig ({current} av {total} byte)."));
    }
    if model.exists() {
        let _ = fs::remove_file(&temp);
    } else {
        fs::rename(&temp, &model).map_err(|e| format!("Kunde inte lägga modellvikterna i cachen: {e}"))?;
    }
    emit(events, json!({"type":"phase","phase":"model-ready"}));
    Ok(())
}

fn collect_wavs(dir: &FsPath, out: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            collect_wavs(&path, out)?;
        } else if ext(&path.to_string_lossy()) == "wav" {
            out.push(path);
        }
    }
    Ok(())
}

fn separate(root: &FsPath, sid: &str, source_name: &str, events: &Events) -> Result<Vec<String>, String> {
    let dir = sess(root, sid).map_err(|e| e.1)?;
    if !is_audio(source_name) {
        return Err("Välj en WAV-, MP3- eller FLAC-fil".into());
    }
    let source = dir.join(source_name);
    if !source.is_file() {
        return Err("Källfilen finns inte i sessionen.".into());
    }
    prepare_model(events)?;
    emit(events, json!({"type":"phase","phase":"separating"}));

    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_nanos();
    let output_dir = dir.join(format!(".demucs-{}-{nonce}", std::process::id()));
    fs::create_dir(&output_dir).map_err(|e| e.to_string())?;
    let _cleanup = TempOutput(output_dir.clone());
    let engine = executable();
    let mut command = Command::new(&engine);
    command.arg(&source).args(["--model", "htdemucs", "--output"]).arg(&output_dir);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let result = command.output().map_err(|e| format!(
        "Kunde inte starta Demucs-RS ({engine:?}): {e}. Kontrollera att demucs.exe ligger bredvid Völund Audio."
    ))?;
    if !result.status.success() {
        let details = String::from_utf8_lossy(if result.stderr.is_empty() { &result.stdout } else { &result.stderr });
        return Err(format!("Demucs-RS misslyckades: {}", details.trim()));
    }

    let mut generated = Vec::new();
    collect_wavs(&output_dir, &mut generated)?;
    let kinds = ["drums", "bass", "other", "vocals"];
    let mut sources = Vec::new();
    for kind in kinds {
        let found = generated.iter().find(|p| p.file_stem().and_then(|s| s.to_str()) == Some(kind))
            .ok_or_else(|| format!("Demucs-RS avslutades men skapade inte filen {kind}.wav"))?;
        sources.push((kind, found.clone()));
    }

    let source_stem = FsPath::new(source_name).file_stem().unwrap_or_default().to_string_lossy();
    let mut index = 1u32;
    let prefix = loop {
        let candidate = if index == 1 { format!("{source_stem}_demucs") } else { format!("{source_stem}_demucs_{index}") };
        if kinds.iter().all(|kind| !dir.join(format!("{candidate}_{kind}.wav")).exists()) { break candidate; }
        index += 1;
    };
    let mut names = Vec::with_capacity(kinds.len());
    for (kind, path) in sources {
        let name = format!("{prefix}_{kind}.wav");
        fs::copy(path, dir.join(&name)).map_err(|e| e.to_string())?;
        names.push(name);
    }
    Ok(names)
}

pub async fn separate_route(
    State(root): State<PathBuf>,
    Path(sid): Path<String>,
    Json(request): Json<Request>,
) -> Response {
    let (events, receiver) = mpsc::unbounded_channel::<Result<Bytes, Infallible>>();
    tokio::task::spawn_blocking(move || {
        let result = separate(&root, &sid, &fname(&request.stem), &events);
        match result {
            Ok(stems) => emit(&events, json!({"type":"complete","stems":stems})),
            Err(message) => emit(&events, json!({"type":"error","message":message})),
        }
    });
    let stream = UnboundedReceiverStream::new(receiver);
    Response::builder()
        .header(header::CONTENT_TYPE, HeaderValue::from_static("text/event-stream; charset=utf-8"))
        .header(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"))
        .body(Body::from_stream(stream))
        .expect("valid SSE response")
}
