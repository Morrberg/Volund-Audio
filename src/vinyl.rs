//! Conservative offline restoration. Original files and frame counts are preserved.
use crate::{bad,decode,fname,sess,Edit,R};
use axum::{extract::{Path,State},Json,response::Response};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use std::{f64::consts::PI,io::Cursor,path::PathBuf};

#[derive(Clone,Default,Deserialize,Serialize)]
pub struct Settings {
    #[serde(default)] pub declick: bool,
    #[serde(default)] pub decrackle: bool,
    #[serde(default="sensitivity_default")] pub sensitivity: f64,
    #[serde(default)] pub derumble: bool,
    #[serde(default="rumble_default")] pub rumble_hz: f64,
    #[serde(default)] pub mono_bass: bool,
}
fn sensitivity_default()->f64{3.}
fn rumble_default()->f64{25.}
impl Settings {
    fn validate(&self)->Result<(),String>{
        if !self.sensitivity.is_finite()||!(1. ..=10.).contains(&self.sensitivity)||![20.,25.].contains(&self.rumble_hz){return Err("Invalid restoration settings".into())}Ok(())
    }
}
#[derive(Clone,Copy)]
struct Section {b:[f64;3],a:[f64;2],z:[f64;2]}
impl Section {
    fn new(sr:u32,hz:f64,q:f64)->Self{
        let w=2.*PI*hz/sr as f64;let c=w.cos();let alpha=w.sin()/(2.*q);let a0=1.+alpha;
        Self{b:[(1.+c)/2./a0,-(1.+c)/a0,(1.+c)/2./a0],a:[-2.*c/a0,(1.-alpha)/a0],z:[0.;2]}
    }
    fn tick(&mut self,x:f64)->f64{let y=self.b[0]*x+self.z[0];self.z[0]=self.b[1]*x-self.a[0]*y+self.z[1];self.z[1]=self.b[2]*x-self.a[1]*y;y}
}
fn declick(y:&mut[f32],sr:u32,ch:usize,sensitivity:f64,crackle:bool)->usize{
    let frames=y.len()/ch;if frames<16{return 0}let radius=(sr as usize/500).max(8);
    let max_width=((sr as f64*if crackle{0.00015}else{0.0005}).round() as usize).max(1);
    let factor=if crackle{14.-sensitivity}else{24.-1.8*sensitivity};
    let floor=if crackle{0.00008}else{0.0003};let mut count=0;
    for c in 0..ch {
        let residual=|i:usize|->f64{if i==0||i+1>=frames {0.}else{(y[i*ch+c] as f64-(y[(i-1)*ch+c] as f64+y[(i+1)*ch+c] as f64)*0.5).abs()}};
        let mut sum=0.;let mut left=0;let mut right=0;let mut ranges=Vec::new();let mut begin=None;
        for i in 1..frames-1 {
            let a=i.saturating_sub(radius);let b=(i+radius+1).min(frames);
            while right<b{sum+=residual(right);right+=1}while left<a{sum-=residual(left);left+=1}
            // Exclude the candidate and its immediate neighbours from the noise estimate.
            let ca=i.saturating_sub(2).max(a);let cb=(i+3).min(b);
            let excluded=(ca..cb).map(&residual).sum::<f64>();
            let background=((sum-excluded).max(0.)/(b-a-(cb-ca)).max(1) as f64).max(floor);
            let flagged=residual(i)>factor*background;
            if flagged&&begin.is_none(){begin=Some(i)}
            if !flagged {if let Some(start)=begin.take(){if i-start<=max_width+2&&start>=2&&i+1<frames {ranges.push((start,i));}}}
        }
        // Linear interpolation is bounded by intact neighbours and cannot overshoot.
        for (a,b) in ranges {
            let l=y[(a-1)*ch+c];let r=y[b*ch+c];let d=(b-a+1) as f32;
            for i in a..b{y[i*ch+c]=l+(r-l)*(i-a+1) as f32/d;}count+=1;
        }
    }count
}
pub fn apply(y:&mut[f32],sr:u32,ch:usize,settings:Option<&Settings>)->Result<usize,String>{
    let Some(s)=settings else{return Ok(0)};s.validate()?;
    if sr<8000||ch==0||ch>8||y.len()%ch!=0||y.iter().any(|v|!v.is_finite()){return Err("Invalid audio for restoration".into())}
    let mut count=0;if s.declick{count+=declick(y,sr,ch,s.sensitivity,false)}if s.decrackle{count+=declick(y,sr,ch,s.sensitivity,true)}
    if s.derumble {
        let mut filters=vec![[Section::new(sr,s.rumble_hz,0.541196100146197),Section::new(sr,s.rumble_hz,1.306562964876377)];ch];
        for frame in y.chunks_exact_mut(ch){for c in 0..ch{let v=filters[c][0].tick(frame[c] as f64);frame[c]=filters[c][1].tick(v) as f32;}}
    }
    if s.mono_bass&&ch==2{
        let mut side_filter=Section::new(sr,120.,std::f64::consts::FRAC_1_SQRT_2);
        for f in y.chunks_exact_mut(2){let mid=(f[0] as f64+f[1] as f64)*0.5;let side=side_filter.tick((f[0] as f64-f[1] as f64)*0.5);f[0]=(mid+side) as f32;f[1]=(mid-side) as f32;}
    }Ok(count)
}

#[derive(Deserialize)]
pub struct Request {
    pub stem:String,
    #[serde(default)] pub edits:Vec<Edit>,
    #[serde(default)] pub restoration:Option<Settings>,
    #[serde(default="threshold_default")] pub threshold_db:f64,
    #[serde(default="gap_default")] pub gap_seconds:f64,
    #[serde(default)] pub markers:Vec<f64>,
}
fn threshold_default()->f64{-45.}
fn gap_default()->f64{1.5}
fn render(root:&std::path::Path,sid:&str,r:&Request)->Result<(Vec<f32>,u32,usize),String>{
    let dir=sess(root,sid).map_err(|e|e.1)?;let(mut y,sr,ch)=decode(&dir.join(fname(&r.stem)))?;
    apply(&mut y,sr,ch,r.restoration.as_ref())?;let(y,ch)=crate::mastering::render_edits(y,sr,ch,&r.edits)?;Ok((y,sr,ch))
}
pub fn find_markers(y:&[f32],sr:u32,ch:usize,db:f64,gap:f64)->Result<Vec<f64>,String>{
    if !db.is_finite()||!(-90. ..=-15.).contains(&db)||!gap.is_finite()||!(0.5..=10.).contains(&gap){return Err("Invalid gap settings".into())}
    let hop=(sr as usize/50).max(1);let threshold=10f64.powf(db/20.);let mut quiet=None;let mut markers=Vec::new();let duration=y.len() as f64/ch as f64/sr as f64;
    for (index,block) in y.chunks(hop*ch).enumerate(){
        let rms=(0..ch).map(|c|{let mut sum=0.;let mut n=0;for f in block.chunks_exact(ch){sum+=(f[c] as f64).powi(2);n+=1}(sum/n.max(1) as f64).sqrt()}).fold(0f64,f64::max);
        let time=index as f64*hop as f64/sr as f64;
        if rms<threshold{if quiet.is_none(){quiet=Some(time)}}else if let Some(a)=quiet.take(){let m=(a+time)*0.5;if a>0.&&time-a>=gap&&m>=5.&&duration-m>=5.&&markers.last().is_none_or(|last|m-last>=5.){markers.push(m)}}
    }Ok(markers)
}
pub async fn markers(State(root):State<PathBuf>,Path(sid):Path<String>,Json(r):Json<Request>)->R<Json<Value>>{
    let result=tokio::task::spawn_blocking(move||->Result<Value,String>{let(y,sr,ch)=render(&root,&sid,&r)?;Ok(json!({"markers":find_markers(&y,sr,ch,r.threshold_db,r.gap_seconds)?}))}).await.map_err(bad)?.map_err(bad)?;Ok(Json(result))
}
fn float_wav(y:&[f32],sr:u32,ch:usize)->Result<Vec<u8>,String>{
    let mut bytes=Cursor::new(Vec::new());{let mut w=hound::WavWriter::new(&mut bytes,hound::WavSpec{channels:ch as u16,sample_rate:sr,bits_per_sample:32,sample_format:hound::SampleFormat::Float}).map_err(|e|e.to_string())?;for v in y{w.write_sample(*v).map_err(|e|e.to_string())?}w.finalize().map_err(|e|e.to_string())?;}Ok(bytes.into_inner())
}
pub async fn split(State(root):State<PathBuf>,Path(sid):Path<String>,Json(r):Json<Request>)->R<Response>{
    let bytes=tokio::task::spawn_blocking(move||->Result<Vec<u8>,String>{
        let(y,sr,ch)=render(&root,&sid,&r)?;let frames=y.len()/ch;let duration=frames as f64/sr as f64;
        if r.markers.len()>500||r.markers.iter().any(|m|!m.is_finite()||*m<=0.||*m>=duration)||r.markers.windows(2).any(|m|m[1]<=m[0]){return Err("Invalid track markers".into())}
        let mut bounds=vec![0usize];bounds.extend(r.markers.iter().map(|m|(m*sr as f64).round() as usize));bounds.push(frames);
        if bounds.windows(2).any(|w|w[1]<=w[0]){return Err("Empty track".into())}
        let mut z=zip::ZipWriter::new(Cursor::new(Vec::new()));let name=fname(&r.stem);let base=std::path::Path::new(&name).file_stem().unwrap_or_default().to_string_lossy();
        for (i,b) in bounds.windows(2).enumerate(){crate::mastering::zip_entry(&mut z,&format!("{base} - {:02}.wav",i+1),&float_wav(&y[b[0]*ch..b[1]*ch],sr,ch)?)?;}
        crate::mastering::zip_entry(&mut z,"Tracks.json",&serde_json::to_vec_pretty(&json!({"source":r.stem,"markers":r.markers,"restoration":r.restoration,"format":"32-bit float WAV","note":"Contiguous split; no audio removed or overlapping. Check playback headroom."})).map_err(|e|e.to_string())?)?;
        z.finish().map(|b|b.into_inner()).map_err(|e|e.to_string())
    }).await.map_err(bad)?.map_err(bad)?;
    crate::mastering::save_or_download(bytes,"Vinyl-tracks.zip".into(),"zip",true).await
}

#[cfg(test)]
mod tests{
 use super::*;
 fn settings()->Settings{Settings{sensitivity:3.,rumble_hz:25.,..Default::default()}}
 #[test]fn clicks_repaired_and_clean_tone_preserved(){let sr=48000;let mut y:Vec<f32>=(0..sr).map(|i|(i as f64*2.*PI*440./sr as f64).sin() as f32*0.1).collect();let original=y.clone();let mut s=settings();s.declick=true;assert_eq!(apply(&mut y,sr,1,Some(&s)).unwrap(),0);assert_eq!(y,original);y[24000]=0.95;assert!(apply(&mut y,sr,1,Some(&s)).unwrap()>0);assert!((y[24000]-original[24000]).abs()<0.01);assert_eq!(y.len(),original.len());}
 #[test]fn finds_gaps_and_keeps_edges(){let sr=1000;let mut y=vec![0.1;20000];y[8000..10000].fill(0.);assert_eq!(find_markers(&y,sr,1,-45.,1.5).unwrap(),vec![9.]);y.fill(0.);assert!(find_markers(&y,sr,1,-45.,1.5).unwrap().is_empty());}
 #[test]fn rumble_attenuation_and_mono_mid_preservation(){let sr=48000;let mut low:Vec<f32>=(0..sr*2).map(|i|(2.*PI*5.*i as f64/sr as f64).sin() as f32).collect();let mut s=settings();s.derumble=true;apply(&mut low,sr,1,Some(&s)).unwrap();let rms=(low[sr as usize..].iter().map(|v|(*v as f64).powi(2)).sum::<f64>()/sr as f64).sqrt();assert!(rms<0.01);let mut stereo=vec![0.1;2000];let before=stereo.clone();s.derumble=false;s.mono_bass=true;apply(&mut stereo,sr,2,Some(&s)).unwrap();assert_eq!(stereo,before);}
}
