//! Album loudness matching uses one constant gain per song, never a second compressor or EQ.
use crate::{bad,decode,sess,R};
use crate::mastering::{self as audio,SideRequest,Metrics};
use axum::{extract::{Path,State},response::Response,Json};
use serde::{Deserialize,Serialize};
use std::{path::{Path as FsPath,PathBuf},io::Cursor};
use serde_json::{json,Value};
#[derive(Clone,Deserialize,Serialize)]
pub struct Settings {pub enabled:bool,pub target_lufs:Option<f64>,pub ceiling_db:f64}
#[derive(Serialize)]
pub struct TrackPlan {pub stem:String,pub input:Metrics,pub matched:bool,pub gain_db:f64,pub expected_lufs:Option<f64>,pub expected_peak_db:Option<f64>,pub warnings:Vec<String>}
#[derive(Serialize)]
pub struct Plan {pub reference_lufs:Option<f64>,pub requested_lufs:Option<f64>,pub effective_lufs:Option<f64>,pub ceiling_db:f64,pub common_attenuation_db:f64,pub enabled:bool,pub tracks:Vec<TrackPlan>,pub warnings:Vec<String>}
fn calculate(r:&SideRequest,metrics:Vec<Metrics>)->Result<Plan,String>{
    let (enabled,manual,ceiling)=r.level_matching.as_ref().map(|s|(s.enabled,s.target_lufs,s.ceiling_db)).unwrap_or((false,None,-1.));
    if !ceiling.is_finite()||!(-12. ..=-0.1).contains(&ceiling)||manual.is_some_and(|v|!v.is_finite()||!(-30. ..=-6.).contains(&v)){return Err("Ogiltigt LUFS-mål eller toppgräns".into())}
    let mut levels:Vec<f64>=r.tracks.iter().zip(&metrics).filter(|(t,_)|t.match_level).filter_map(|(_,m)|m.lufs).collect();levels.sort_by(f64::total_cmp);
    let reference=if levels.is_empty(){None}else{let n=levels.len();Some(if n%2==0{(levels[n/2-1]+levels[n/2])/2.}else{levels[n/2]})};
    let target=if enabled&&reference.is_some(){manual.or(reference)}else{None};let mut shift=0f64;let mut tracks=Vec::new();
    for (t,input) in r.tracks.iter().zip(metrics){
        let matched=enabled&&t.match_level&&input.lufs.is_some();
        let boost=if matched{target.zip(input.lufs).map_or(0.,|(a,b)|a-b)}else{0.};let g=boost+t.offset_db;
        // One common reduction preserves all requested inter-track differences.
        shift=shift.min(12.-g);if let Some(peak)=input.true_peak_db{shift=shift.min(ceiling-0.15-peak-g)}
        let warnings=if input.lufs.is_none(){vec!["unmeasurable".into()]}else{vec![]};
        tracks.push(TrackPlan{stem:t.stem.clone(),expected_lufs:None,expected_peak_db:None,input,matched,gain_db:g,warnings});
    }
    for t in &mut tracks{t.gain_db+=shift;t.expected_lufs=t.input.lufs.map(|v|v+t.gain_db);t.expected_peak_db=t.input.true_peak_db.map(|v|v+t.gain_db)}
    let mut warnings=Vec::new();if shift< -0.01{warnings.push("common_reduction".into())}if enabled&&reference.is_none(){warnings.push("no_reference".into())}
    Ok(Plan{reference_lufs:reference,requested_lufs:target,effective_lufs:target.map(|v|v+shift),ceiling_db:ceiling,common_attenuation_db:-shift,enabled,tracks,warnings})
}
pub(crate) fn plan(dir:&FsPath,r:&SideRequest)->Result<Plan,String>{
    audio::validate_sides(r)?;let mut metrics=Vec::new();
    for t in &r.tracks{let (y,sr,ch)=decode(&dir.join(&t.stem))?;let y=audio::stereo_resample(&y,sr,ch,r.sample_rate)?;metrics.push(audio::measure(&y,r.sample_rate)?)}
    calculate(r,metrics)
}
pub async fn analyze(State(root):State<PathBuf>,Path(sid):Path<String>,Json(r):Json<SideRequest>)->R<Json<Plan>>{
    let dir=sess(&root,&sid)?;let result=tokio::task::spawn_blocking(move||plan(&dir,&r)).await.map_err(bad)?.map_err(bad)?;Ok(Json(result))
}
fn export_zip(dir:&FsPath,r:&SideRequest)->Result<Vec<u8>,String>{
    let plan=plan(dir,r)?;let mut zip=zip::ZipWriter::new(Cursor::new(Vec::new()));let mut outputs=Vec::<Value>::new();
    for (i,t) in r.tracks.iter().enumerate(){
        let (y,sr,ch)=decode(&dir.join(&t.stem))?;let mut y=audio::stereo_resample(&y,sr,ch,r.sample_rate)?;
        let g=audio::gain(plan.tracks[i].gain_db) as f32;for sample in &mut y{*sample*=g}audio::quantize(&mut y,r.bits);
        let output=audio::measure(&y,r.sample_rate)?;if output.true_peak_db.is_some_and(|v|v>plan.ceiling_db){return Err("Albumets toppgräns överskreds".into())}
        let title=std::path::Path::new(&t.stem).file_stem().unwrap_or_default().to_string_lossy();let name=format!("{:02} - {} - album.wav",i+1,audio::safe_name(&title));
        let bytes=crate::metadata::apply(&dir.join(&t.stem),audio::wav(&y,r.sample_rate,r.bits,false)?,None)?;
        audio::zip_entry(&mut zip,&name,&bytes)?;outputs.push(json!({"file":name,"source":t.stem,"gain_db":plan.tracks[i].gain_db,"metrics":output}));
    }
    audio::zip_entry(&mut zip,"Album-report.json",&serde_json::to_vec_pretty(&json!({"version":1,"request":r,"plan":plan,"outputs":outputs})).map_err(|e|e.to_string())?)?;
    zip.finish().map(|z|z.into_inner()).map_err(|e|e.to_string())
}
pub async fn export(State(root):State<PathBuf>,Path(sid):Path<String>,Json(r):Json<SideRequest>)->R<Response>{
    let dir=sess(&root,&sid)?;let name=format!("{} - album.zip",audio::safe_name(&r.title));
    let bytes=tokio::task::spawn_blocking(move||export_zip(&dir,&r)).await.map_err(bad)?.map_err(bad)?;audio::save_or_download(bytes,name,"zip",true).await
}
#[cfg(test)] mod tests{
    use super::*;
    fn request()->SideRequest{serde_json::from_value(json!({"title":"Album","sample_rate":44100,"bits":24,"level_matching":{"enabled":true,"target_lufs":null,"ceiling_db":-1.},"tracks":[{"stem":"Loud.wav","side":"A","gap":0.},{"stem":"Quiet.wav","side":"B","gap":2.}]})).unwrap()}
    fn sine(a:f64)->Vec<f32>{(0..44100*3).flat_map(|i|{let v=(a*(i as f64*2.*std::f64::consts::PI*1000./44100.).sin()) as f32;[v,v]}).collect()}
    #[test]fn match_preserves_dynamics_stereo_and_matches_exports(){
        let dir=std::env::temp_dir().join(format!("album-test-{}",std::process::id()));std::fs::create_dir_all(&dir).unwrap();let r=request();
        for (t,a) in r.tracks.iter().zip([0.4,0.1]){std::fs::write(dir.join(&t.stem),audio::wav(&sine(a),44100,24,true).unwrap()).unwrap()}
        let p=plan(&dir,&r).unwrap();assert!((p.tracks[0].expected_lufs.unwrap()-p.tracks[1].expected_lufs.unwrap()).abs()<0.01);
        let bytes=export_zip(&dir,&r).unwrap();let mut zip=zip::ZipArchive::new(Cursor::new(bytes)).unwrap();let report:Value={let mut file=zip.by_name("Album-report.json").unwrap();serde_json::from_reader(&mut file).unwrap()};
        let levels:Vec<f64>=report["outputs"].as_array().unwrap().iter().map(|o|o["metrics"]["lufs"].as_f64().unwrap()).collect();assert!((levels[0]-levels[1]).abs()<0.02);
        let mut data=Vec::new();std::io::Read::read_to_end(&mut zip.by_name("01 - Loud - album.wav").unwrap(),&mut data).unwrap();let mut wav=hound::WavReader::new(Cursor::new(data)).unwrap();assert_eq!(wav.spec().bits_per_sample,24);let samples:Vec<i32>=wav.samples::<i32>().map(Result::unwrap).collect();assert_eq!(samples.len(),44100*3*2);assert!(samples.chunks_exact(2).all(|f|(f[0]-f[1]).abs()<=2));
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]fn exceptions_offsets_silence_and_headroom_use_common_gain(){
        let mut r=request();let m=vec![audio::measure(&sine(0.4),44100).unwrap(),audio::measure(&sine(0.1),44100).unwrap()];r.level_matching.as_mut().unwrap().target_lufs=Some(-6.);
        let p=calculate(&r,m.clone()).unwrap();assert!(p.common_attenuation_db>0.);assert!((p.tracks[0].expected_lufs.unwrap()-p.tracks[1].expected_lufs.unwrap()).abs()<0.01);assert!(p.tracks.iter().all(|t|t.expected_peak_db.unwrap()<=-1.));
        r.tracks[1].match_level=false;let p=calculate(&r,m.clone()).unwrap();assert!(!p.tracks[1].matched);assert!((p.tracks[1].gain_db+p.common_attenuation_db).abs()<1e-6);
        r.tracks[1].match_level=true;r.tracks[1].offset_db=-3.;let p=calculate(&r,m).unwrap();assert!((p.tracks[0].expected_lufs.unwrap()-p.tracks[1].expected_lufs.unwrap()-3.).abs()<0.01);
        let p=calculate(&r,vec![audio::measure(&sine(0.),44100).unwrap(),audio::measure(&sine(0.1),44100).unwrap()]).unwrap();assert!(!p.tracks[0].matched);assert!(p.tracks[0].expected_lufs.is_none());
        r.tracks[0].match_level=false;r.tracks[1].match_level=false;let p=calculate(&r,vec![audio::measure(&sine(0.4),44100).unwrap(),audio::measure(&sine(0.1),44100).unwrap()]).unwrap();assert!(p.effective_lufs.is_none());assert!(p.tracks.iter().all(|t|!t.matched));
        r.level_matching.as_mut().unwrap().ceiling_db=f64::NAN;assert!(calculate(&r,vec![]).is_err());
    }
}
