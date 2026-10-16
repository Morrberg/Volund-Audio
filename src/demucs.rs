//! Optional external Demucs-RS runner. Model weights are downloaded by the engine on first use.
use crate::{bad, ext, fname, is_audio, sess, R};
use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path as FsPath, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

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

fn collect_wavs(dir: &FsPath, out: &mut Vec<PathBuf>) -> R<()> {
    for entry in fs::read_dir(dir).map_err(bad)? {
        let entry = entry.map_err(bad)?;
        let path = entry.path();
        if entry.file_type().map_err(bad)?.is_dir() {
            collect_wavs(&path, out)?;
        } else if ext(&path.to_string_lossy()) == "wav" {
            out.push(path);
        }
    }
    Ok(())
}

fn separate(root: &FsPath, sid: &str, source_name: &str) -> R<Vec<String>> {
    let dir = sess(root, sid)?;
    if !is_audio(source_name) {
        return Err(bad("Välj en WAV-, MP3- eller FLAC-fil"));
    }
    let source = dir.join(source_name);
    if !source.is_file() {
        return Err((StatusCode::NOT_FOUND, "källfilen finns inte i sessionen".into()));
    }

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(bad)?
        .as_nanos();
    let output_dir = dir.join(format!(".demucs-{}-{nonce}", std::process::id()));
    fs::create_dir(&output_dir).map_err(bad)?;
    let _cleanup = TempOutput(output_dir.clone());
    let engine = executable();
    let result = Command::new(&engine)
        .arg(&source)
        .args(["--model", "htdemucs", "--output"])
        .arg(&output_dir)
        .output()
        .map_err(|e| bad(format!(
            "Kunde inte starta Demucs-RS ({engine:?}): {e}. Kontrollera att demucs.exe ligger bredvid Völund Audio."
        )))?;
    if !result.status.success() {
        let details = String::from_utf8_lossy(if result.stderr.is_empty() {
            &result.stdout
        } else {
            &result.stderr
        });
        return Err(bad(format!("Demucs-RS misslyckades: {}", details.trim())));
    }

    let mut generated = Vec::new();
    collect_wavs(&output_dir, &mut generated)?;
    let kinds = ["drums", "bass", "other", "vocals"];
    let mut sources = Vec::new();
    for kind in kinds {
        let found = generated
            .iter()
            .find(|p| p.file_stem().and_then(|s| s.to_str()) == Some(kind))
            .ok_or_else(|| bad(format!("Demucs-RS avslutades men skapade inte filen {kind}.wav")))?;
        sources.push((kind, found.clone()));
    }

    let source_stem = FsPath::new(source_name)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let mut index = 1u32;
    let prefix = loop {
        let candidate = if index == 1 {
            format!("{source_stem}_demucs")
        } else {
            format!("{source_stem}_demucs_{index}")
        };
        if kinds.iter().all(|kind| !dir.join(format!("{candidate}_{kind}.wav")).exists()) {
            break candidate;
        }
        index += 1;
    };

    let mut names = Vec::with_capacity(kinds.len());
    for (kind, path) in sources {
        let name = format!("{prefix}_{kind}.wav");
        fs::copy(path, dir.join(&name)).map_err(bad)?;
        names.push(name);
    }
    Ok(names)
}

pub async fn separate_route(
    State(root): State<PathBuf>,
    Path(sid): Path<String>,
    Json(request): Json<Request>,
) -> R<Json<Value>> {
    let stems = tokio::task::spawn_blocking(move || separate(&root, &sid, &fname(&request.stem)))
        .await
        .map_err(bad)??;
    Ok(Json(json!({ "stems": stems })))
}
