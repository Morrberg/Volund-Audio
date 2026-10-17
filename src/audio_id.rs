//! Local Chromaprint generation with explicit, optional AcoustID lookup.
//! Audio stays on this machine; only the compact fingerprint and duration are
//! sent when the user explicitly requests an online lookup.

use axum::{extract::{Path, State}, Json};
use chromaprint::{Algorithm, Fingerprinter};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{path::PathBuf, sync::{Mutex, OnceLock}, thread, time::{Duration, Instant}};

use crate::{bad, decode, fname, sess, R};

#[derive(Deserialize)]
pub struct Request {
    stem: String,
    #[serde(default)] markers: Vec<f64>,
    client_key: String,
}

fn fingerprint(audio: &[f32], sample_rate: u32, channels: usize) -> Result<String, String> {
    if channels == 0 || channels > u16::MAX as usize || audio.len() % channels != 0 {
        return Err("Invalid audio channel layout".into());
    }
    let pcm: Vec<i16> = audio.iter().map(|sample| {
        (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16
    }).collect();
    let mut printer = Fingerprinter::new(Algorithm::default());
    printer.start(sample_rate, channels as u16).map_err(|e| e.to_string())?;
    printer.feed(&pcm).map_err(|e| e.to_string())?;
    printer.finish().map_err(|e| e.to_string())?;
    Ok(printer.encode())
}

fn lookup(client_key: &str, duration: f64, fingerprint: &str) -> Result<Value, String> {
    let duration_text = format!("{}", duration.round().max(1.0) as u64);
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout_read(Duration::from_secs(12))
        .build();
    let response = agent.post("https://api.acoustid.org/v2/lookup")
        .send_form(&[
            ("client", client_key),
            ("duration", duration_text.as_str()),
            ("fingerprint", fingerprint),
            ("meta", "recordings+releasegroups+releases+tracks+isrcs"),
        ])
        .map_err(|e| format!("AcoustID request failed: {e}"))?
        .into_string().map_err(|e| e.to_string())?;
    let parsed: Value = serde_json::from_str(&response).map_err(|e| e.to_string())?;
    if parsed["status"] != "ok" {
        return Err(parsed["error"]["message"].as_str().unwrap_or("AcoustID returned an error").to_string());
    }
    let mut matches = Vec::new();
    for result in parsed["results"].as_array().into_iter().flatten().take(5) {
        let recordings: Vec<Value> = result["recordings"].as_array().into_iter().flatten().take(5).map(|recording| {
            let artists = recording["artists"].as_array().into_iter().flatten()
                .filter_map(|artist| artist["name"].as_str()).collect::<Vec<_>>();
            let albums = recording["releasegroups"].as_array().into_iter().flatten()
                .filter_map(|album| album["title"].as_str()).collect::<Vec<_>>();
            let isrcs = recording["isrcs"].as_array().into_iter().flatten()
                .filter_map(Value::as_str).collect::<Vec<_>>();
            let releases: Vec<Value> = recording["releasegroups"].as_array().into_iter().flatten()
                .flat_map(|group| group["releases"].as_array().into_iter().flatten())
                .take(10)
                .filter_map(|release| {
                    let id = release["id"].as_str()?;
                    let mediums: Vec<Value> = release["mediums"].as_array().into_iter().flatten().map(|medium| json!({
                        "format": medium["format"],
                        "position": medium["position"],
                        "track_count": medium["track_count"],
                    })).collect();
                    Some(json!({
                        "release_id": id,
                        "title": release["title"],
                        "country": release["country"],
                        "date": release["date"],
                        "medium_count": release["medium_count"],
                        "track_count": release["track_count"],
                        "mediums": mediums,
                    }))
                }).collect();
            json!({
                "recording_id": recording["id"],
                "title": recording["title"],
                "artists": artists,
                "albums": albums,
                "releases": releases,
                "isrcs": isrcs,
                "duration": recording["duration"],
            })
        }).collect();
        matches.push(json!({"score": result["score"], "recordings": recordings}));
    }
    Ok(json!({"matches": matches}))
}

/// Explicit online lookup for the sections defined by the current vinyl markers.
pub async fn identify(
    State(root): State<PathBuf>,
    Path(sid): Path<String>,
    Json(request): Json<Request>,
) -> R<Json<Value>> {
    if request.client_key.trim().len() < 8 || request.client_key.len() > 128 {
        return Err(bad("Enter a valid AcoustID application key"));
    }
    if request.markers.len() > 99 || request.markers.iter().any(|m| !m.is_finite() || *m <= 0.0)
        || request.markers.windows(2).any(|w| w[1] <= w[0]) {
        return Err(bad("Invalid vinyl markers"));
    }
    let path = sess(&root, &sid)?.join(fname(&request.stem));
    let (audio, sample_rate, channels) = tokio::task::spawn_blocking(move || decode(&path))
        .await.map_err(bad)?.map_err(bad)?;
    let duration = audio.len() as f64 / channels as f64 / sample_rate as f64;
    if request.markers.iter().any(|m| *m >= duration) { return Err(bad("A marker is outside the audio")); }

    let result = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let mut bounds = vec![0.0];
        bounds.extend(request.markers.iter().copied());
        bounds.push(duration);
        let mut tracks = Vec::new();
        let mut lookups = 0usize;
        for (index, pair) in bounds.windows(2).enumerate() {
            let start = (pair[0] * sample_rate as f64).round() as usize;
            let end = (pair[1] * sample_rate as f64).round() as usize;
            let frames = end.saturating_sub(start);
            let seconds = frames as f64 / sample_rate as f64;
            if seconds < 10.0 {
                tracks.push(json!({"track":index+1,"start":pair[0],"end":pair[1],"skipped":"Segment shorter than 10 seconds"}));
                continue;
            }
            if lookups > 0 { thread::sleep(Duration::from_millis(400)); }
            let from = start.saturating_mul(channels);
            let to = end.saturating_mul(channels).min(audio.len());
            let encoded = fingerprint(&audio[from..to], sample_rate, channels)?;
            let response = lookup(&request.client_key, seconds, &encoded)?;
            tracks.push(json!({
                "track":index+1,
                "start":(pair[0]*1000.0).round()/1000.0,
                "end":(pair[1]*1000.0).round()/1000.0,
                "fingerprint_frames": frames,
                "matches": response["matches"],
            }));
            lookups += 1;
        }
        Ok(json!({"provider":"AcoustID / MusicBrainz", "audio_uploaded":false, "tracks":tracks}))
    }).await.map_err(bad)?.map_err(bad)?;
    Ok(Json(result))
}

static MUSICBRAINZ_RATE_LIMIT: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();

fn fetch_release(release_id: &str) -> Result<Value, String> {
    let rate_lock = MUSICBRAINZ_RATE_LIMIT.get_or_init(|| Mutex::new(None));
    let mut last_request = rate_lock.lock().map_err(|_| "MusicBrainz rate limiter is unavailable")?;
    if let Some(previous) = *last_request {
        let elapsed = previous.elapsed();
        if elapsed < Duration::from_secs(1) {
            thread::sleep(Duration::from_secs(1) - elapsed);
        }
    }
    *last_request = Some(Instant::now());
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout_read(Duration::from_secs(15))
        .build();
    let url = format!("https://musicbrainz.org/ws/2/release/{release_id}?inc=recordings+media+artist-credits&fmt=json");
    let body = agent.get(&url)
        .set("User-Agent", "VolundAudio/0.95 (https://github.com/Morrberg/Volund-Audio)")
        .call()
        .map_err(|e| format!("MusicBrainz request failed: {e}"))?
        .into_string().map_err(|e| e.to_string())?;
    let release: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    let credits = |value: &Value| -> String {
        value.as_array().into_iter().flatten().map(|credit| {
            let name = credit["name"].as_str().or_else(|| credit["artist"]["name"].as_str()).unwrap_or("");
            let joinphrase = credit["joinphrase"].as_str().unwrap_or("");
            format!("{name}{joinphrase}")
        }).collect::<String>()
    };
    let artist = credits(&release["artist-credit"]);
    let mediums: Vec<Value> = release["media"].as_array().into_iter().flatten().map(|medium| {
        let tracks: Vec<Value> = medium["tracks"].as_array().into_iter().flatten().map(|track| {
            let track_artist = credits(&track["artist-credit"]);
            json!({
                "number": track["number"],
                "position": track["position"],
                "title": track["title"],
                "length_ms": track["length"],
                "artist": track_artist,
                "recording_id": track["recording"]["id"],
            })
        }).collect();
        json!({
            "format": medium["format"],
            "position": medium["position"],
            "track_count": medium["track-count"],
            "tracks": tracks,
        })
    }).collect();
    Ok(json!({
        "release_id": release["id"],
        "title": release["title"],
        "artist": artist,
        "date": release["date"],
        "country": release["country"],
        "barcode": release["barcode"],
        "mediums": mediums,
    }))
}

/// Fetch the complete track listing for a release chosen from an AcoustID match.
pub async fn release(Path(release_id): Path<String>) -> R<Json<Value>> {
    let valid_uuid = release_id.len() == 36 && release_id.chars().enumerate().all(|(i, c)| {
        if [8, 13, 18, 23].contains(&i) { c == '-' } else { c.is_ascii_hexdigit() }
    });
    if !valid_uuid { return Err(bad("Invalid MusicBrainz release ID")); }
    let data = tokio::task::spawn_blocking(move || fetch_release(&release_id))
        .await.map_err(bad)?.map_err(bad)?;
    Ok(Json(data))
}
