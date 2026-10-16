//! Völund Audio: eget Windows-fönster, lokal ljudbehandling.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
//! Originalfiler skrivs aldrig över; export ger en ny WAV-fil.
mod metadata;
mod mastering;
mod album;
#[cfg(target_os="windows")]
static ENGLISH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
#[cfg(target_os="windows")]
fn ui_text(sv: &'static str, en: &'static str) -> &'static str {if ENGLISH.load(std::sync::atomic::Ordering::Relaxed){en}else{sv}}


use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{header, HeaderName, StatusCode},
    response::Html,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs,
    io::{Cursor, Read, Write},
    path::{Path as FsPath, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
    process::Command,
};
use symphonia::core::{
    audio::SampleBuffer, codecs::DecoderOptions, formats::FormatOptions, io::MediaSourceStream,
    meta::MetadataOptions, probe::Hint,
};

const INDEX: &str = include_str!("../static/index.html");
type R<T> = Result<T, (StatusCode, String)>;

fn bad<E: ToString>(e: E) -> (StatusCode, String) { (StatusCode::BAD_REQUEST, e.to_string()) }
fn fname(n: &str) -> String {
    FsPath::new(n).file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}
fn ext(n: &str) -> String {
    FsPath::new(n).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase()
}
fn is_audio(n: &str) -> bool { matches!(ext(n).as_str(), "wav" | "mp3" | "flac") }
fn r2(x: f32) -> f64 { ((x * 100.0).round() / 100.0) as f64 }
fn sess(root: &FsPath, sid: &str) -> R<PathBuf> {
    if sid.is_empty() || !sid.chars().all(|c| c.is_ascii_alphanumeric()) { return Err(bad("ogiltigt sid")); }
    let p = root.join(sid);
    if p.is_dir() { Ok(p) } else { Err((StatusCode::NOT_FOUND, "okänd session".into())) }
}

/// Avkoda WAV/MP3/FLAC till interleaved f32. Returnerar (samples, samplingsfrekvens, kanaler).
fn decode(p: &FsPath) -> Result<(Vec<f32>, u32, usize), String> {
    let f = fs::File::open(p).map_err(|e| e.to_string())?;
    let mss = MediaSourceStream::new(Box::new(f), Default::default());
    let mut hint = Hint::new();
    hint.with_extension(&ext(&p.to_string_lossy()));
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| e.to_string())?;
    let mut fmt = probed.format;
    let track = fmt.default_track().ok_or("inget ljudspår")?;
    let id = track.id;
    let mut dec = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| e.to_string())?;
    let (mut out, mut sr, mut ch) = (Vec::new(), 0u32, 1usize);
    while let Ok(pkt) = fmt.next_packet() {
        if pkt.track_id() != id { continue; }
        if let Ok(buf) = dec.decode(&pkt) {
            sr = buf.spec().rate;
            ch = buf.spec().channels.count();
            let mut sb = SampleBuffer::<f32>::new(buf.capacity() as u64, *buf.spec());
            sb.copy_interleaved_ref(buf);
            out.extend_from_slice(sb.samples());
        }
    }
    if sr == 0 { return Err("kunde inte avkoda filen".into()); }
    Ok((out, sr, ch))
}

async fn upload(State(root): State<PathBuf>, mut mp: Multipart) -> R<Json<Value>> {
    let sid = format!("{:x}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis());
    let d = root.join(&sid);
    fs::create_dir_all(&d).map_err(bad)?;
    let mut project = Value::Null;
    while let Some(f) = mp.next_field().await.map_err(bad)? {
        let name = fname(f.file_name().unwrap_or(""));
        let raw = f.bytes().await.map_err(bad)?;
        if matches!(ext(&name).as_str(), "zip" | "vcleaner") {
            let mut z = zip::ZipArchive::new(Cursor::new(raw)).map_err(bad)?;
            for i in 0..z.len() {
                let mut e = z.by_index(i).map_err(bad)?;
                let n = fname(e.name());
                if n == "project.json" {
                    let mut b = String::new();e.read_to_string(&mut b).map_err(bad)?;
                    project = serde_json::from_str(&b).map_err(bad)?;
                } else if is_audio(&n) {
                    let mut b = Vec::new();
                    e.read_to_end(&mut b).map_err(bad)?;
                    fs::write(d.join(&n), b).map_err(bad)?;
                }
            }
        } else if is_audio(&name) {
            fs::write(d.join(&name), &raw).map_err(bad)?;
        }
    }
    let mut stems: Vec<String> = fs::read_dir(&d).map_err(bad)?
        .filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    stems.sort();
    if stems.is_empty() { return Err(bad("Inga ljudfiler hittades")); }
    let vocal = stems.iter().find(|s| { let l = s.to_lowercase(); l.contains("vocal") || l.contains("sång") }).cloned();
    Ok(Json(json!({"sid": sid, "stems": stems, "vocal": vocal, "project": project})))
}

async fn stem(State(root): State<PathBuf>, Path((sid, name)): Path<(String, String)>)
    -> R<([(HeaderName, &'static str); 1], Vec<u8>)> {
    let p = sess(&root, &sid)?.join(fname(&name));
    let ct = match ext(&name).as_str() { "mp3" => "audio/mpeg", "flac" => "audio/flac", _ => "audio/wav" };
    Ok(([(header::CONTENT_TYPE, ct)], fs::read(p).map_err(bad)?))
}

/// Energibaserad aktivitetsdetektering + kontextregler. Returnerar förslag, ändrar inget.
#[derive(Deserialize)]
struct SeparateReq {
    stem: String,
    python: String,
    #[serde(default = "default_demucs_device")]
    device: String,
}
fn default_demucs_device() -> String { "cuda".into() }

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

async fn separate(State(root): State<PathBuf>, Path(sid): Path<String>, Json(req): Json<SeparateReq>)
    -> R<Json<Value>> {
    let stems = tokio::task::spawn_blocking(move || -> R<Vec<String>> {
        let dir = sess(&root, &sid)?;
        let source_name = fname(&req.stem);
        if !is_audio(&source_name) { return Err(bad("Välj en WAV-, MP3- eller FLAC-fil")); }
        let source = dir.join(&source_name);
        if !source.is_file() { return Err((StatusCode::NOT_FOUND, "källfilen finns inte i sessionen".into())); }
        let python = req.python.trim();
        if python.is_empty() { return Err(bad("Ange sökvägen till Python i Demucs-miljön")); }
        let device = match req.device.as_str() { "cuda" => "cuda", "cpu" => "cpu", _ => return Err(bad("enheten måste vara cuda eller cpu")) };
        let run = SystemTime::now().duration_since(UNIX_EPOCH).map_err(bad)?.as_millis();
        let output_dir = dir.join(format!(".demucs-{run}"));
        fs::create_dir_all(&output_dir).map_err(bad)?;
        let result = Command::new(python)
            .args(["-m", "demucs", "-d", device, "-n", "htdemucs", "--segment", "6",
                "--shifts", "0", "--overlap", "0.25", "--float32", "--out"])
            .arg(&output_dir)
            .arg(&source)
            .output()
            .map_err(bad)?;
        if !result.status.success() {
            let _ = fs::remove_dir_all(&output_dir);
            let details = String::from_utf8_lossy(if result.stderr.is_empty() { &result.stdout } else { &result.stderr });
            return Err(bad(format!("Demucs misslyckades: {}", details.trim())));
        }
        let mut generated = Vec::new();
        collect_wavs(&output_dir, &mut generated)?;
        let kinds = ["drums", "bass", "other", "vocals"];
        let mut sources = Vec::new();
        for kind in kinds {
            let found = generated.iter().find(|p| p.file_stem().and_then(|s| s.to_str()) == Some(kind))
                .ok_or_else(|| bad(format!("Demucs avslutades men skapade inte filen {kind}.wav")))?;
            sources.push((kind, found.clone()));
        }
        let source_stem = FsPath::new(&source_name).file_stem().unwrap_or_default().to_string_lossy();
        let mut index = 1u32;
        let prefix = loop {
            let candidate = if index == 1 { format!("{source_stem}_demucs") } else { format!("{source_stem}_demucs_{index}") };
            if kinds.iter().all(|kind| !dir.join(format!("{candidate}_{kind}.wav")).exists()) { break candidate; }
            index += 1;
        };
        let mut names = Vec::new();
        for (kind, path) in sources {
            let name = format!("{prefix}_{kind}.wav");
            fs::copy(path, dir.join(&name)).map_err(bad)?;
            names.push(name);
        }
        fs::remove_dir_all(&output_dir).map_err(bad)?;
        Ok(names)
    }).await.map_err(bad)??;
    Ok(Json(json!({"stems": stems})))
}

async fn analyze(State(root): State<PathBuf>, Path((sid, name)): Path<(String, String)>) -> R<Json<Value>> {
    let (y, sr, ch) = decode(&sess(&root, &sid)?.join(fname(&name))).map_err(bad)?;
    let m: Vec<f32> = y.chunks(ch).map(|c| c.iter().sum::<f32>() / ch as f32).collect();
    let total = m.len() as f32 / sr as f32;
    let hop = ((sr as f32 * 0.02) as usize).max(1);
    let n = m.len() / hop;
    let db: Vec<f32> = (0..n).map(|i| {
        let s = &m[i * hop..(i + 1) * hop];
        20.0 * ((s.iter().map(|v| v * v).sum::<f32>() / hop as f32).sqrt() + 1e-9).log10()
    }).collect();
    let thr = (db.iter().cloned().fold(f32::MIN, f32::max) - 45.0).max(-60.0);
    let mut segs: Vec<(f32, f32)> = vec![];
    let mut i = 0;
    while i < n {
        if db[i] > thr {
            let mut j = i;
            while j < n && db[j] > thr { j += 1; }
            let (a, b) = (i as f32 * 0.02, j as f32 * 0.02);
            match segs.last_mut() { Some(l) if a - l.1 < 0.4 => l.1 = b, _ => segs.push((a, b)) }
            i = j;
        } else { i += 1; }
    }
    let ok = |g: Option<f32>| g.map_or(false, |g| g >= 1.5);
    let mut out = vec![];
    for (k, &(a, b)) in segs.iter().enumerate() {
        let (first, last) = (k == 0, k + 1 == segs.len());
        let gb = if first { None } else { Some(a - segs[k - 1].1) };
        let ga = if last { None } else { Some(segs[k + 1].0 - b) };
        let dur = b - a;
        let near = a < 20.0 || b > total - 20.0;
        let inner = (first && ok(ga)) || (last && ok(gb)) || (ok(gb) && ok(ga));
        if dur < 4.0 && near && inner {
            let gap = [gb, ga].iter().flatten().cloned().fold(f32::MAX, f32::min);
            let conf = (0.4 + 0.3 * (gap / 5.0).min(1.0) + 0.3 * (1.0 - dur / 4.0)).clamp(0.0, 0.95);
            let pos = if a < 20.0 { "i början" } else { "i slutet" };
            out.push(json!({"start": r2(a), "end": r2(b), "confidence": r2(conf),
                "desc": format!("Kort isolerat vokalt ljud {pos} ({dur:.1} s, {gap:.1} s tystnad intill)")}));
        }
    }
    Ok(Json(json!({"duration": total, "suggestions": out})))
}

#[derive(Clone, Default, Deserialize, Serialize)]
struct Edit { kind: String, start: f32, end: f32, #[serde(default)] fade: f32,
    #[serde(default)] pan_from: f32, #[serde(default)] pan_to: f32, #[serde(default)] pan_cycles: u16 }
#[derive(Deserialize)]
struct ExportReq { stem: String, edits: Vec<Edit>, #[serde(default)] metadata: Option<metadata::Changes> }

async fn export(State(root): State<PathBuf>, Path(sid): Path<String>, Json(r): Json<ExportReq>)
    -> R<([(HeaderName, String); 2], Vec<u8>)> {
    let name = fname(&r.stem);
    let (y, sr, ch) = decode(&sess(&root, &sid)?.join(&name)).map_err(bad)?;
    let (y, ch) = mastering::render_edits(y, sr, ch, &r.edits).map_err(bad)?;
    let spec = hound::WavSpec { channels: ch as u16, sample_rate: sr, bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float };
    let mut buf = Cursor::new(Vec::new());
    {
        let mut w = hound::WavWriter::new(&mut buf, spec).map_err(bad)?;
        for s in &y { w.write_sample(*s).map_err(bad)?; }
        w.finalize().map_err(bad)?;
    }
    let stem_name = FsPath::new(&name).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    Ok(([(header::CONTENT_TYPE, "audio/wav".to_string()),
         (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{stem_name}_cleaned.wav\""))],
        metadata::apply(&sess(&root, &sid)?.join(&name), buf.into_inner(), r.metadata.as_ref()).map_err(bad)?))
}

async fn serve(ready: std::sync::mpsc::Sender<String>) {
    let dir = std::env::temp_dir().join("vocal-cleaner");
    fs::create_dir_all(&dir).expect("kan inte skapa arbetsmapp");
    let app = Router::new()
        .route("/", get(|| async { Html(INDEX) }))
        .route("/icon.svg", get(|| async { ([(header::CONTENT_TYPE, "image/svg+xml")], include_str!("../assets/icon.svg")) }))
        .route("/api/upload", post(upload))
        .route("/api/stem/:sid/:name", get(stem))
        .route("/api/demucs/:sid", post(separate))
        .route("/api/analyze/:sid/:name", get(analyze))
        .route("/api/metadata/:sid/:name", get(inspect_metadata))
        .route("/api/export/:sid", post(export))
        .route("/api/save/:sid", post(save))
        .route("/api/project/:sid", post(save_project))
        .route("/api/master/analyze/:sid", post(mastering::analyze))
        .route("/api/master/preview/:sid", post(mastering::preview))
        .route("/api/master/export/:sid", post(mastering::batch))
        .route("/api/master/save/:sid", post(mastering::save_batch))
        .route("/api/master/single/:sid", post(mastering::single))
        .route("/api/master/premaster/:sid", post(mastering::premaster))
        .route("/api/master/sides/:sid", post(mastering::sides))
        .route("/api/album/analyze/:sid", post(album::analyze))
        .route("/api/album/export/:sid", post(album::export))
        .layer(DefaultBodyLimit::max(2 << 30))
        .with_state(dir);
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", l.local_addr().unwrap());
    println!("Völund Audio kör på {url}  (Ctrl+C för att avsluta)");
    ready.send(url).expect("kan inte starta gränssnittet");
    axum::serve(l, app).await.unwrap();
}

async fn save(state: State<PathBuf>, sid: Path<String>, req: Json<ExportReq>) -> R<Json<Value>> {
    #[cfg(target_os = "windows")]
    {
        let name = fname(&req.0.stem);
        let suggested = format!("{}_cleaned.wav", FsPath::new(&name).file_stem().unwrap_or_default().to_string_lossy());
        let path = tokio::task::spawn_blocking(move || rfd::FileDialog::new()
            .set_title(ui_text("Spara redigerat spår", "Save edited track"))
            .add_filter("WAV-ljud", &["wav"])
            .set_file_name(&suggested).save_file()).await.map_err(bad)?;
        let Some(path) = path else { return Ok(Json(json!({"cancelled": true}))); };
        // Stoppa överskrivning av importerade original även om användaren väljer arbetsmappen.
        let input = sess(&state.0, &sid.0)?.join(&name);
        if fs::canonicalize(&path).ok() == fs::canonicalize(&input).ok() {
            return Err(bad("Välj ett nytt filnamn så att originalet bevaras"));
        }
        let (_, bytes) = export(state, sid, req).await?;
        fs::write(&path, bytes).map_err(bad)?;
        Ok(Json(json!({"cancelled": false, "name": path.file_name().unwrap_or_default().to_string_lossy()})))
    }
    #[cfg(not(target_os = "windows"))]
    { let _ = (state, sid, req); Err(bad("Använd Exportera sångspår i webbläsarversionen")) }
}

fn main() {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        tokio::runtime::Runtime::new().expect("kan inte starta ljudserver").block_on(serve(tx));
    });
    let url = rx.recv().expect("ljudservern kunde inte starta");
    // CI uses the same embedded UI and API without needing an interactive desktop.
    if let Ok(path) = std::env::var("VOCAL_CLEANER_TEST_URL_FILE") {
        fs::write(path, &url).expect("kan inte lämna serveradressen till UI-testet");
    }
    if std::env::var("VOCAL_CLEANER_SERVER_ONLY").as_deref() == Ok("1") {
        loop { std::thread::park(); }
    }
    #[cfg(target_os = "windows")]
    {
        use tao::{event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().with_title("Völund Audio")
            .with_inner_size(tao::dpi::LogicalSize::new(1320.0, 860.0))
            .with_window_icon(tao::window::Icon::from_rgba(include_bytes!("../assets/icon.rgba").to_vec(), 64, 64).ok())
            .build(&event_loop).expect("kan inte skapa programfönster");
        let dirty = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let ipc_dirty = dirty.clone();
        let _webview = wry::WebViewBuilder::new(&window).with_url(&url)
            .with_ipc_handler(move |request| { match request.body().as_str() {
                "language:en"=>ENGLISH.store(true,std::sync::atomic::Ordering::Relaxed),
                "language:sv"=>ENGLISH.store(false,std::sync::atomic::Ordering::Relaxed),
                value=>ipc_dirty.store(value == "dirty", std::sync::atomic::Ordering::Relaxed),
            } })
            .build().expect("Kan inte starta WebView2. Installera Microsoft Edge WebView2 Runtime.");
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            if let Event::WindowEvent { event: WindowEvent::CloseRequested, .. } = event {
                let close = !dirty.load(std::sync::atomic::Ordering::Relaxed) ||
                    rfd::MessageDialog::new().set_title(ui_text("Osparat projekt", "Unsaved project"))
                    .set_description(ui_text("Projektet har osparade ändringar. Stäng ändå? Välj Nej och Spara projekt för att behålla dem.", "The project has unsaved changes. Close anyway? Choose No and Save project to keep your work."))
                    .set_buttons(rfd::MessageButtons::YesNo).show() == rfd::MessageDialogResult::Yes;
                if close { *control_flow = ControlFlow::Exit; }
            }
        });
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = open::that(&url);
        loop { std::thread::park(); }
    }
}

fn project_zip(dir: &FsPath, project: &Value) -> R<Vec<u8>> {
    let mut z = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    z.start_file("project.json", options).map_err(bad)?;
    z.write_all(&serde_json::to_vec(project).map_err(bad)?).map_err(bad)?;
    for entry in fs::read_dir(dir).map_err(bad)? {
        let entry = entry.map_err(bad)?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if is_audio(&name) {
            z.start_file(name, options).map_err(bad)?;
            std::io::copy(&mut fs::File::open(entry.path()).map_err(bad)?, &mut z).map_err(bad)?;
        }
    }
    Ok(z.finish().map_err(bad)?.into_inner())
}
async fn save_project(State(root): State<PathBuf>, Path(sid): Path<String>, Json(project): Json<Value>)
    -> R<axum::response::Response> {
    use axum::response::IntoResponse;
    let bytes = project_zip(&sess(&root, &sid)?, &project)?;
    if std::env::var("VOCAL_CLEANER_SERVER_ONLY").as_deref() == Ok("1") {
        return Ok(([(header::CONTENT_TYPE, "application/zip")], bytes).into_response());
    }
    #[cfg(target_os = "windows")]
    {
        let path = tokio::task::spawn_blocking(|| rfd::FileDialog::new()
            .set_title(ui_text("Spara Völund Audio-projekt", "Save Völund Audio project")).add_filter("Völund Audio", &["vcleaner"])
            .set_file_name("Volund-Audio.vcleaner").save_file()).await.map_err(bad)?;
        let Some(path) = path else { return Ok(Json(json!({"cancelled": true})).into_response()); };
        fs::write(path, bytes).map_err(bad)?;
        Ok(Json(json!({"cancelled": false})).into_response())
    }
    #[cfg(not(target_os = "windows"))]
    { Ok(([(header::CONTENT_TYPE, "application/zip"),
        (header::CONTENT_DISPOSITION, "attachment; filename=Volund-Audio.vcleaner")], bytes).into_response()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn project_keeps_original_audio_and_edit_manifest() {
        let dir = std::env::temp_dir().join(format!("vocal-cleaner-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("Vocals.wav"), b"original audio bytes").unwrap();
        fs::write(dir.join("Drums.wav"), b"drums").unwrap();
        let manifest = json!({"version":1,"tracks":{"Vocals.wav":{"edits":[{"kind":"mute","start":1,"end":2,"fade":0.01}]}}});
        let bytes = project_zip(&dir, &manifest).unwrap();
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        let mut audio = Vec::new();archive.by_name("Vocals.wav").unwrap().read_to_end(&mut audio).unwrap();
        assert_eq!(audio, b"original audio bytes");
        let mut text = String::new();archive.by_name("project.json").unwrap().read_to_string(&mut text).unwrap();
        assert_eq!(serde_json::from_str::<Value>(&text).unwrap(), manifest);
        assert!(archive.by_name("Drums.wav").is_ok());
        fs::remove_dir_all(dir).unwrap();
    }
}

async fn inspect_metadata(State(root): State<PathBuf>, Path((sid, name)): Path<(String, String)>) -> R<Json<Value>> {
    let path = sess(&root, &sid)?.join(fname(&name));
    let info = tokio::task::spawn_blocking(move || metadata::inspect(&path)).await.map_err(bad)?.map_err(bad)?;
    Ok(Json(info))
}
