//! Deterministic offline stereo mastering. Every profile starts from the unmastered mix.
use crate::{bad, decode, fname, sess, Edit, R};
use axum::{extract::{Path, State}, http::{header, HeaderName}, response::{IntoResponse, Response}, Json};
use ebur128::{EbuR128, Mode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::{HashSet, VecDeque}, f64::consts::PI, io::{Cursor, Write}, path::Path as FsPath};

#[derive(Clone, Deserialize, Serialize)]
pub struct MixTrack {
    pub stem: String,
    #[serde(default)] pub edits: Vec<Edit>,
    #[serde(default)] pub gain_db: f64,
    #[serde(default)] pub mute: bool,
    #[serde(default)] pub solo: bool,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Profile {
    pub id: String,
    pub sample_rate: u32,
    pub bits: u16,
    pub target_lufs: Option<f64>,
    pub ceiling_db: f64,
    pub bass_db: f64,
    pub mid_db: f64,
    pub treble_db: f64,
    pub compression: f64,
    pub mono_bass_hz: f64,
    pub highpass_hz: f64,
    pub limiter: bool,
}
impl Profile {
    fn validate(&self) -> Result<(), String> {
        if !["Spotify", "Vinyl", "CD", "Kassett"].contains(&self.id.as_str()) { return Err("Okänd masteringprofil".into()); }
        if ![44100, 48000, 88200, 96000].contains(&self.sample_rate) || ![16,24].contains(&self.bits) { return Err("Välj 44,1/48/88,2/96 kHz och 16/24 bit".into()); }
        if self.id == "CD" && (self.sample_rate != 44100 || self.bits != 16) { return Err("CD kräver 16 bit / 44,1 kHz".into()); }
        for (v,a,b) in [(self.ceiling_db,-12.,-0.1),(self.bass_db,-6.,6.),(self.mid_db,-6.,6.),(self.treble_db,-6.,6.),(self.compression,0.,100.),(self.mono_bass_hz,0.,250.),(self.highpass_hz,0.,80.)] {
            if !v.is_finite() || v < a || v > b { return Err("Masteringinställning utanför tillåtet intervall".into()); }
        }
        if self.target_lufs.is_some_and(|v| !v.is_finite() || !(-30. ..=-6.).contains(&v)) { return Err("LUFS-målet måste vara mellan −30 och −6".into()); }
        Ok(())
    }
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Request {
    pub title: String,
    #[serde(default)] pub artist: String,
    #[serde(default)] pub album: String,
    pub tracks: Vec<MixTrack>,
    pub profiles: Vec<Profile>,
    #[serde(default)] pub mix_edits: Vec<MixEdit>,
}
/// Times refer to the current mix after all preceding operations.
#[derive(Clone, Deserialize, Serialize)]
pub struct MixEdit { pub kind: String, pub start: f64, pub end: f64 }
fn edit_mix(mut audio: Vec<f32>, sr: u32, edits: &[MixEdit]) -> Result<Vec<f32>, String> {
    if edits.len()>1000 {return Err("Too many mix edits".into())}
    for e in edits {
        let frames=audio.len()/2;
        if !e.start.is_finite() || !e.end.is_finite() || e.start<0. || e.end<=e.start || e.end>frames as f64/sr as f64+1.0/sr as f64 || !["cut","keep"].contains(&e.kind.as_str()) {return Err("Invalid mix selection".into())}
        let a=((e.start*sr as f64).round() as usize).min(frames);
        let b=((e.end*sr as f64).round() as usize).min(frames);
        if b<=a || (e.kind=="cut" && b-a==frames) {return Err("The mix must contain audio".into())}
        let joins=if e.kind=="cut" {audio.drain(a*2..b*2);vec![a]} else {audio=audio[a*2..b*2].to_vec();vec![0,audio.len()/2]};
        // Short fades preserve exact timing; no overlapping or shortening at joins.
        let count=audio.len()/2;let edge=(sr as usize/200).min(count/2);
        for join in joins {for i in 0..edge {
            let gain=i as f32/edge as f32;
            for c in 0..2 {if join+i<count {audio[(join+i)*2+c]*=gain}if join>i {audio[(join-1-i)*2+c]*=gain}}
        }}
    }
    Ok(audio)
}
fn request_mix(dir:&FsPath,r:&Request,sr:u32)->Result<Vec<f32>,String>{edit_mix(mix(dir,&r.tracks,sr)?,sr,&r.mix_edits)}
#[derive(Clone, Serialize)]
pub struct Metrics {
    pub lufs: Option<f64>,
    pub true_peak_db: Option<f64>,
    pub lra: Option<f64>,
    pub crest_db: Option<f64>,
    pub correlation: Option<f64>,
    pub bass_side_ratio: f64,
    pub treble_ratio: f64,
    pub duration: f64,
}
#[derive(Serialize)]
pub struct Report {
    pub profile: String,
    pub input: Metrics,
    pub output: Metrics,
    pub requested_lufs: Option<f64>,
    pub gain_db: f64,
    pub compressor_reduction_db: f64,
    pub limiter_reduction_db: f64,
    pub safety_attenuation_db: f64,
    pub warnings: Vec<String>,
}
fn db(v: f64) -> Option<f64> { if v > 1e-15 { Some(20.*v.log10()) } else { None } }
pub(crate) fn gain(db: f64) -> f64 { 10f64.powf(db/20.) }
fn finite(v: Result<f64, ebur128::Error>) -> Option<f64> { v.ok().filter(|x| x.is_finite()) }

pub fn measure(y: &[f32], sr: u32) -> Result<Metrics, String> {
    let mut meter = EbuR128::new(2, sr, Mode::I | Mode::LRA | Mode::TRUE_PEAK | Mode::HISTOGRAM).map_err(|e|e.to_string())?;
    meter.add_frames_f32(y).map_err(|e|e.to_string())?;
    // Flush the interpolation filter's tail without affecting the loudness integration.
    let lufs=finite(meter.loudness_global());let lra=finite(meter.loudness_range());
    meter.add_frames_f32(&[0.;128]).map_err(|e|e.to_string())?;
    let peak=(0..2).filter_map(|c|meter.true_peak(c).ok()).fold(0f64,f64::max);
    let (mut ll,mut rr,mut lr,mut squares,mut sample_peak)=(0.,0.,0.,0.,0f64);
    let (mut low_l,mut low_r,mut high_l,mut high_r)=(0.,0.,0.,0.);
    let (mut bass_mid,mut bass_side,mut treble)=(0.,0.,0.);
    let a=1.-(-2.*PI*120./sr as f64).exp();let h=1.-(-2.*PI*6000./sr as f64).exp();
    for frame in y.chunks_exact(2) {
        let (l,r)=(frame[0] as f64,frame[1] as f64);
        ll+=l*l;rr+=r*r;lr+=l*r;squares+=l*l+r*r;sample_peak=sample_peak.max(l.abs()).max(r.abs());
        low_l+=a*(l-low_l);low_r+=a*(r-low_r);high_l+=h*(l-high_l);high_r+=h*(r-high_r);
        bass_mid+=((low_l+low_r)*0.5).powi(2);bass_side+=((low_l-low_r)*0.5).powi(2);
        treble+=(l-high_l).powi(2)+(r-high_r).powi(2);
    }
    let rms=(squares/y.len().max(1) as f64).sqrt();
    Ok(Metrics{lufs,true_peak_db:db(peak),lra,crest_db:if rms>1e-15 { db(sample_peak/rms) } else {None},correlation:if ll*rr>1e-20 {Some((lr/(ll*rr).sqrt()).clamp(-1.,1.))}else{None},bass_side_ratio:bass_side/(bass_mid+bass_side+1e-30),treble_ratio:treble/(squares+1e-30),duration:y.len() as f64/2./sr as f64})
}

/// Mono is promoted only when panning is requested. Untouched samples keep their original stereo position.
pub fn render_edits(mut y: Vec<f32>, sr: u32, ch: usize, edits: &[Edit]) -> Result<(Vec<f32>,usize), String> {
    let mono=ch==1;
    let channels=if edits.iter().any(|e|e.kind=="pan") {
        if !(1..=2).contains(&ch) {return Err("Panorering stöder mono och stereo".into())}
        if mono {y=y.into_iter().flat_map(|v|[v,v]).collect();}2
    } else {ch};
    apply_edits(&mut y,sr,channels,mono,edits)?;Ok((y,channels))
}
fn pan_frame(l:f64,r:f64,p:f64,mono:bool)->(f64,f64) {
    if p==0. {return (l,r)}
    if mono && p<=-1. {return ((l+r)*0.5*2f64.sqrt(),0.)}if mono && p>=1. {return (0.,(l+r)*0.5*2f64.sqrt())}
    if mono {let angle=(p+1.)*PI/4.;let source=(l+r)*0.5;return (source*2f64.sqrt()*angle.cos(),source*2f64.sqrt()*angle.sin())}
    if p<=-1. {return (l+r,0.)}if p>=1. {return (0.,r+l)}
    if p<0. {let angle=(p+1.)*PI/2.;(l+r*angle.cos(),r*angle.sin())}
    else {let angle=p*PI/2.;(l*angle.cos(),r+l*angle.sin())}
}
fn apply_edits(y: &mut [f32], sr: u32, ch: usize, mono: bool, edits: &[Edit]) -> Result<(), String> {
    if ch==0 {return Err("Ljudet saknar kanaler".into())}
    let frames=y.len()/ch;
    let mut pan_base:Option<Vec<f32>>=None;
    let pans=edits.iter().enumerate().filter(|(i,e)|e.kind=="pan"&&!edits[i+1..].iter().any(|later|later.kind=="pan"&&later.start==e.start&&later.end==e.end)).map(|(_,e)|e);
    for e in edits.iter().filter(|e|e.kind!="pan").chain(pans) {
        if !e.start.is_finite() || !e.end.is_finite() || !e.fade.is_finite() || e.start<0. || e.end<e.start || !(0. ..=2.).contains(&e.fade) || !["mute","fadein","fadeout","pan"].contains(&e.kind.as_str()) { return Err("Ogiltig spårredigering".into()); }
        if e.kind=="pan" && (!e.pan_from.is_finite() || !e.pan_to.is_finite() || !(-1. ..=1.).contains(&e.pan_from) || !(-1. ..=1.).contains(&e.pan_to) || e.pan_cycles>32) {return Err("Ogiltig panorering".into())}
        let a=((e.start as f64*sr as f64).round() as usize).min(frames);let b=((e.end as f64*sr as f64).round() as usize).min(frames);let n=b-a;if n==0 {continue}
        if e.kind=="pan" {
            let edge=((e.fade as f64*sr as f64).round() as usize).min((n-1)/2);
            let base=pan_base.get_or_insert_with(||y.to_vec());
            for i in 0..n {
                let t=if n>1 {i as f64/(n-1) as f64}else{0.};
                let t=if e.pan_cycles>0 {(1.-(2.*PI*e.pan_cycles as f64*t).cos())*0.5}else{t};
                let p=e.pan_from as f64+(e.pan_to as f64-e.pan_from as f64)*t;
                let weight=if edge>0 {(i as f64/edge as f64).min((n-1-i) as f64/edge as f64).min(1.)}else{1.};
                let j=(a+i)*2;let (l,r)=(y[j] as f64,y[j+1] as f64);let (pl,pr)=pan_frame(base[j] as f64,base[j+1] as f64,p,mono);
                y[j]=(l+(pl-l)*weight) as f32;y[j+1]=(r+(pr-r)*weight) as f32;
            }
            continue;
        }
        let mut at=|i:usize,g:f32|{for c in 0..ch {y[i*ch+c]*=g}};
        match e.kind.as_str() {
            "fadein"=>for i in 0..n {at(a+i,i as f32/n as f32)},
            "fadeout"=>for i in 0..n {at(a+i,1.-i as f32/n as f32)},
            _=>{let f=((e.fade*sr as f32).round() as usize).min(a).min(frames-b);for i in 0..f {at(a-f+i,1.-i as f32/f as f32);at(b+i,i as f32/f as f32)}for i in a..b {at(i,0.)}}
        }
    }
    Ok(())
}
/// Symmetric windowed-sinc resampling. No latency, fixed rounded output duration.
pub fn stereo_resample(y: &[f32], from: u32, ch: usize, to: u32) -> Result<Vec<f32>, String> {
    if !(1..=2).contains(&ch) || from<8000 || from>192000 || y.iter().any(|v|!v.is_finite()) {return Err("Mastering stöder ändligt mono/stereoljud, 8–192 kHz".into())}
    let input=y.len()/ch;let output=(input as f64*to as f64/from as f64).round() as usize;
    if output>to as usize*60*60 {return Err("En mix får vara högst 60 minuter".into())}
    let mut result=vec![0.;output*2];
    if from==to {for (i,f) in y.chunks_exact(ch).enumerate(){result[2*i]=f[0];result[2*i+1]=f[ch-1]}return Ok(result)}
    let cutoff=(to as f64/from as f64).min(1.)*0.94;let radius=(32./cutoff).ceil() as isize;
    // Precompute fractional phases: avoids transcendental functions per output sample.
    let phases=1024;let taps=(radius*2) as usize;let mut kernel=vec![0.;phases*taps];
    for phase in 0..phases {
        let frac=phase as f64/phases as f64;let mut sum=0.;
        for tap in 0..taps {let d=(tap as isize-radius+1) as f64-frac;let x=d*cutoff;let w=if d.abs()<radius as f64 {(if x.abs()<1e-12 {cutoff}else{(PI*x).sin()/(PI*d)})*(0.5+0.5*(PI*d/radius as f64).cos())}else{0.};kernel[phase*taps+tap]=w;sum+=w}
        for w in &mut kernel[phase*taps..(phase+1)*taps] {*w/=sum}
    }
    for i in 0..output {
        let pos=i as f64*from as f64/to as f64;let center=pos.floor() as isize;let phase=((pos-pos.floor())*phases as f64) as usize;
        let (mut l,mut r)=(0.,0.);
        for (tap,w) in kernel[phase*taps..(phase+1)*taps].iter().enumerate(){let index=center+tap as isize-radius+1;if index>=0 && (index as usize)<input {let j=index as usize*ch;l+=y[j] as f64*w;r+=y[j+ch-1] as f64*w}}
        result[2*i]=l as f32;result[2*i+1]=r as f32;
    }
    Ok(result)
}
fn mix(dir: &FsPath, tracks: &[MixTrack], sr: u32) -> Result<Vec<f32>, String> {
    if tracks.is_empty() || tracks.len()>64 {return Err("Välj 1–64 stems".into())}
    let any_solo=tracks.iter().any(|t|t.solo&&!t.mute);let mut seen=HashSet::new();let mut out=Vec::<f32>::new();
    for t in tracks {
        if fname(&t.stem)!=t.stem || !crate::is_audio(&t.stem) || !seen.insert(&t.stem) {return Err("Ogiltigt eller dubblerat spårnamn".into())}
        if !t.gain_db.is_finite() || !(-60. ..=12.).contains(&t.gain_db) {return Err("Spårnivån måste vara −60 till +12 dB".into())}
        let (y,rate,ch)=decode(&dir.join(&t.stem))?;let (y,ch)=render_edits(y,rate,ch,&t.edits)?;
        let y=stereo_resample(&y,rate,ch,sr)?;
        // Muted/solo-excluded tracks still determine the full mix duration.
        out.resize(out.len().max(y.len()),0.);
        if t.mute || (any_solo&&!t.solo) {continue}
        let g=gain(t.gain_db) as f32;for (a,b) in out.iter_mut().zip(y) {*a+=b*g}
    }
    if out.is_empty() || out.iter().any(|v|!v.is_finite()) {return Err("Mixen saknar giltigt ljud".into())}
    Ok(out)
}

#[derive(Clone,Copy)]
struct Biquad {b:[f64;3],a:[f64;2],z:[f64;2]}
impl Biquad {
    fn tick(&mut self,x:f64)->f64 {let y=self.b[0]*x+self.z[0];self.z[0]=self.b[1]*x-self.a[0]*y+self.z[1];self.z[1]=self.b[2]*x-self.a[1]*y;y}
    fn new(kind:&str,rate:u32,freq:f64,db:f64)->Self {
        let w=2.*PI*freq/rate as f64;let (s,c)=w.sin_cos();let a=10f64.powf(db/40.);let alpha=s/(2.*0.7071067811865476);let beta=2.*a.sqrt()*alpha;
        let (b,a0,a1,a2)=match kind {
            "low"=>([a*((a+1.)-(a-1.)*c+beta),2.*a*((a-1.)-(a+1.)*c),a*((a+1.)-(a-1.)*c-beta)],(a+1.)+(a-1.)*c+beta,-2.*((a-1.)+(a+1.)*c),(a+1.)+(a-1.)*c-beta),
            "high"=>([a*((a+1.)+(a-1.)*c+beta),-2.*a*((a-1.)+(a+1.)*c),a*((a+1.)+(a-1.)*c-beta)],(a+1.)-(a-1.)*c+beta,2.*((a-1.)-(a+1.)*c),(a+1.)-(a-1.)*c-beta),
            "peak"=>([1.+alpha*a,-2.*c,1.-alpha*a],1.+alpha/a,-2.*c,1.-alpha/a),
            _=>([(1.+c)/2.,-(1.+c),(1.+c)/2.],1.+alpha,-2.*c,1.-alpha),
        };
        Self{b:b.map(|v|v/a0),a:[a1/a0,a2/a0],z:[0.;2]}
    }
}
fn tone(y:&mut [f32],p:&Profile) {
    for (kind,f,db) in [("hp",p.highpass_hz,0.),("low",120.,p.bass_db),("peak",1000.,p.mid_db),("high",6000.,p.treble_db)] {
        if (kind=="hp"&&f<=0.) || (kind!="hp"&&db==0.) {continue}
        let mut filters=[Biquad::new(kind,p.sample_rate,f,db);2];for frame in y.chunks_exact_mut(2) {for c in 0..2 {frame[c]=filters[c].tick(frame[c] as f64) as f32}}
    }
    if p.mono_bass_hz>0. {
        let a=1.-(-2.*PI*p.mono_bass_hz/p.sample_rate as f64).exp();let mut low=0.;
        for f in y.chunks_exact_mut(2){let mid=(f[0] as f64+f[1] as f64)*0.5;let side=(f[0] as f64-f[1] as f64)*0.5;low+=a*(side-low);let side=side-low;f[0]=(mid+side) as f32;f[1]=(mid-side) as f32}
    }
}
fn compress(y:&mut [f32],p:&Profile)->f64 {
    if p.compression==0. {return 0.}
    let amount=p.compression/100.;let ratio=1.+amount*1.5;let threshold=-12.-amount*8.;let knee=6.;
    let attack=(-1./(0.025*p.sample_rate as f64)).exp();let release=(-1./(0.150*p.sample_rate as f64)).exp();let detector=(-1./(0.010*p.sample_rate as f64)).exp();
    let (mut power,mut reduction,mut max_reduction)=(0.,0.,0f64);
    for f in y.chunks_exact_mut(2) {
        let instantaneous=(f[0] as f64).powi(2).max((f[1] as f64).powi(2));power=detector*power+(1.-detector)*instantaneous;
        let over=10.*power.max(1e-30).log10()-threshold;
        let target=if over<=-knee/2. {0.} else if over>=knee/2. {(1.-1./ratio)*over} else {(1.-1./ratio)*(over+knee/2.).powi(2)/(2.*knee)};
        let coefficient=if target>reduction {attack}else{release};reduction=coefficient*reduction+(1.-coefficient)*target;max_reduction=max_reduction.max(reduction);
        let g=gain(-reduction) as f32;f[0]*=g;f[1]*=g;
    }
    max_reduction
}
fn limit(y:&mut [f32],sr:u32,ceiling:f64)->f64 {
    let frames=y.len()/2;let ahead=(sr as f64*0.005).round() as usize;let ceiling=gain(ceiling);
    let mut maxima=VecDeque::<(usize,f64)>::new();let (mut next,mut g,mut reduction)=(0usize,1f64,0f64);let release=(-1./(0.080*sr as f64)).exp();
    for i in 0..frames {
        while next<frames && next<=i+ahead {
            let v=(y[2*next] as f64).abs().max((y[2*next+1] as f64).abs());
            while maxima.back().is_some_and(|x|x.1<=v) {maxima.pop_back();}maxima.push_back((next,v));next+=1;
        }
        while maxima.front().is_some_and(|x|x.0<i) {maxima.pop_front();}
        let target=(ceiling/maxima.front().map_or(1e-30,|x|x.1).max(1e-30)).min(1.);
        g=if target<g {target}else{release*g+(1.-release)*target};reduction=reduction.max(-20.*g.max(1e-30).log10());
        y[2*i]*=g as f32;y[2*i+1]*=g as f32;
    }
    reduction
}
/// Deterministic TPDF dither at the final integer conversion, never added to exact digital silence.
pub(crate) fn quantize(y:&mut [f32],bits:u16) {
    let scale=((1u32<<(bits-1))-1) as f64;let mut state=0x8ba63e21u32;
    let mut rand=||{state^=state<<13;state^=state>>17;state^=state<<5;state as f64/u32::MAX as f64};
    for s in y {if *s==0. {continue}let noise=rand()-rand();*s=(((*s as f64*scale+noise).round()).clamp(-scale-1.,scale)/scale) as f32}
}
pub(crate) fn wav(y:&[f32],sr:u32,bits:u16,float:bool)->Result<Vec<u8>,String> {
    let mut buffer=Cursor::new(Vec::new());
    {let mut writer=hound::WavWriter::new(&mut buffer,hound::WavSpec{channels:2,sample_rate:sr,bits_per_sample:if float{32}else{bits},sample_format:if float{hound::SampleFormat::Float}else{hound::SampleFormat::Int}}).map_err(|e|e.to_string())?;
    let scale=((1u32<<(bits-1))-1) as f64;for s in y {if float {writer.write_sample(*s).map_err(|e|e.to_string())?}else{writer.write_sample((*s as f64*scale).round().clamp(-scale-1.,scale) as i32).map_err(|e|e.to_string())?}}writer.finalize().map_err(|e|e.to_string())?;}
    Ok(buffer.into_inner())
}
fn process(mix:&[f32],p:&Profile)->Result<(Vec<f32>,Report),String> {
    p.validate()?;let input=measure(mix,p.sample_rate)?;let mut y=mix.to_vec();tone(&mut y,p);let compression=compress(&mut y,p);let toned=measure(&y,p.sample_rate)?;
    let mut warnings=Vec::new();let requested=p.target_lufs.zip(toned.lufs).map_or(0.,|(target,current)|target-current);let boost=requested.clamp(-60.,12.);
    if requested>12. {warnings.push("gain_limited".into())}
    for s in &mut y {*s*=gain(boost) as f32}
    let reduction=if p.limiter {limit(&mut y,p.sample_rate,p.ceiling_db-0.05)}else{0.};
    let mut safety=0.;
    // Measure the rendered, dithered signal too. Ceiling wins over loudness target.
    for _ in 0..3 {
        let peak=measure(&y,p.sample_rate)?.true_peak_db.unwrap_or(-200.);
        if peak>p.ceiling_db-0.05 {let db=p.ceiling_db-0.05-peak;safety-=db;for s in &mut y {*s*=gain(db) as f32}}
        quantize(&mut y,p.bits);
        if measure(&y,p.sample_rate)?.true_peak_db.unwrap_or(-200.)<=p.ceiling_db {break}
    }
    let output=measure(&y,p.sample_rate)?;
    if output.true_peak_db.is_some_and(|v|v>p.ceiling_db+0.001) {return Err("Kunde inte uppfylla profilens true-peak-gräns".into())}
    if input.lufs.is_none(){warnings.push("silent_or_short".into())}
    if reduction>3. {warnings.push("heavy_limiting".into())}
    if p.target_lufs.zip(output.lufs).is_some_and(|(target,actual)|(actual-target).abs()>1.) {warnings.push("target_not_reached".into())}
    if p.id=="Vinyl" {
        warnings.push("vinyl_cutting_review".into());
        if input.bass_side_ratio>0.25 {warnings.push("wide_bass".into())}
        if input.treble_ratio>0.20 {warnings.push("bright_treble".into())}
    }
    if p.id=="Kassett" {warnings.push("cassette_supplier_review".into())}
    if p.id=="Spotify" && output.lufs.is_some_and(|v|v> -14.) && p.ceiling_db> -2. {warnings.push("streaming_headroom".into())}
    Ok((y,Report{profile:p.id.clone(),input,output,requested_lufs:p.target_lufs,gain_db:boost,compressor_reduction_db:compression,limiter_reduction_db:reduction,safety_attenuation_db:safety,warnings}))
}
fn validate(r:&Request)->Result<(),String>{
    if r.title.trim().is_empty() || r.title.len()>200 || r.artist.len()>500 || r.album.len()>500 {return Err("Ange en låttitel (högst 200 byte)".into())}
    if r.profiles.is_empty() || r.profiles.len()>4 {return Err("Välj 1–4 masteringprofiler".into())}
    let mut ids=HashSet::new();for p in &r.profiles {p.validate()?;if !ids.insert(&p.id){return Err("Dubblerad masteringprofil".into())}}
    Ok(())
}
pub(crate) fn safe_name(s:&str)->String {
    let s:String=s.chars().map(|c|if c.is_control()||"/\\:*?\"<>|".contains(c){'_'}else{c}).collect();
    let s=s.trim().trim_matches('.').to_string();if s.is_empty(){"Låt".into()}else{s}
}
fn tagged(y:&[f32],p:&Profile,r:&Request)->Result<Vec<u8>,String> {
    let bytes=wav(y,p.sample_rate,p.bits,false)?;
    let changes=crate::metadata::Changes{clear:true,fields:vec![crate::metadata::Field{key:"TrackTitle".into(),value:r.title.clone()},crate::metadata::Field{key:"TrackArtist".into(),value:r.artist.clone()},crate::metadata::Field{key:"AlbumTitle".into(),value:r.album.clone()}]};
    crate::metadata::apply(FsPath::new(""),bytes,Some(&changes))
}
pub(crate) fn zip_entry(zip:&mut zip::ZipWriter<Cursor<Vec<u8>>>,name:&str,bytes:&[u8])->Result<(),String> {
    zip.start_file(name,zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored)).map_err(|e|e.to_string())?;
    zip.write_all(bytes).map_err(|e|e.to_string())
}
fn render_batch(dir:&FsPath,r:&Request)->Result<Vec<u8>,String> {
    validate(r)?;let mut zip=zip::ZipWriter::new(Cursor::new(Vec::new()));let mut reports=Vec::new();
    // Cache only one sample rate at a time to keep batch memory bounded.
    let mut rate=0;let mut base=Vec::new();
    for p in &r.profiles {
        if rate!=p.sample_rate {base=request_mix(dir,r,p.sample_rate)?;rate=p.sample_rate}
        let (y,report)=process(&base,p)?;
        zip_entry(&mut zip,&format!("{} - mastered - {}.wav",safe_name(&r.title),p.id),&tagged(&y,p,r)?)?;reports.push(report);
    }
    zip_entry(&mut zip,"Mastering-report.json",&serde_json::to_vec_pretty(&json!({"version":1,"request":r,"reports":reports})).map_err(|e|e.to_string())?)?;
    zip.finish().map(|z|z.into_inner()).map_err(|e|e.to_string())
}
pub async fn analyze(State(root):State<std::path::PathBuf>,Path(sid):Path<String>,Json(r):Json<Request>)->R<Json<Value>> {
    let dir=sess(&root,&sid)?;
    let result=tokio::task::spawn_blocking(move||->Result<Value,String>{validate(&r)?;let p=&r.profiles[0];let base=request_mix(&dir,&r,p.sample_rate)?;let (_,report)=process(&base,p)?;Ok(json!(report))}).await.map_err(bad)?.map_err(bad)?;
    Ok(Json(result))
}
pub async fn preview(State(root):State<std::path::PathBuf>,Path(sid):Path<String>,Json(r):Json<Request>)->R<([(HeaderName,String);1],Vec<u8>)> {
    let dir=sess(&root,&sid)?;
    let bytes=tokio::task::spawn_blocking(move||->Result<Vec<u8>,String>{validate(&r)?;let p=&r.profiles[0];let base=request_mix(&dir,&r,p.sample_rate)?;let(y,_)=process(&base,p)?;wav(&y,p.sample_rate,p.bits,true)}).await.map_err(bad)?.map_err(bad)?;
    Ok(([(header::CONTENT_TYPE,"audio/wav".into())],bytes))
}
pub(crate) async fn save_or_download(bytes:Vec<u8>,suggested:String,extension:&'static str,native:bool)->R<Response> {
    #[cfg(target_os="windows")]
    if native && std::env::var("VOCAL_CLEANER_SERVER_ONLY").as_deref()!=Ok("1") {
        let file_name=suggested.clone();let path=tokio::task::spawn_blocking(move||rfd::FileDialog::new().set_title(crate::ui_text("Spara ljudexport", "Save audio export")).add_filter("Export",&[extension]).set_file_name(&file_name).save_file()).await.map_err(bad)?;
        let Some(path)=path else{return Ok(Json(json!({"cancelled":true})).into_response())};
        std::fs::write(&path,bytes).map_err(bad)?;return Ok(Json(json!({"cancelled":false,"name":path.file_name().unwrap_or_default().to_string_lossy()})).into_response());
    }
    let _=(native,extension,&suggested);
    // UTF-8 names are provided by the UI for browser downloads; ASCII fallback avoids invalid headers.
    Ok(([(header::CONTENT_TYPE,if extension=="wav"{"audio/wav"}else{"application/zip"}),(header::CONTENT_DISPOSITION,"attachment; filename=Mastering-export.zip")],bytes).into_response())
}
pub async fn batch(State(root):State<std::path::PathBuf>,Path(sid):Path<String>,Json(r):Json<Request>)->R<Response> {batch_impl(root,sid,r,false).await}
pub async fn save_batch(State(root):State<std::path::PathBuf>,Path(sid):Path<String>,Json(r):Json<Request>)->R<Response> {batch_impl(root,sid,r,true).await}
async fn batch_impl(root:std::path::PathBuf,sid:String,r:Request,native:bool)->R<Response> {
    let dir=sess(&root,&sid)?;let title=safe_name(&r.title);
    let bytes=tokio::task::spawn_blocking(move||render_batch(&dir,&r)).await.map_err(bad)?.map_err(bad)?;
    save_or_download(bytes,format!("{title} - mastered.zip"),"zip",native).await
}
pub async fn single(State(root):State<std::path::PathBuf>,Path(sid):Path<String>,Json(r):Json<Request>)->R<Response> {
    let dir=sess(&root,&sid)?;validate(&r).map_err(bad)?;
    let suggested=format!("{} - mastered - {}.wav",safe_name(&r.title),r.profiles[0].id);
    let bytes=tokio::task::spawn_blocking(move||->Result<Vec<u8>,String>{let p=&r.profiles[0];let base=request_mix(&dir,&r,p.sample_rate)?;let (y,_)=process(&base,p)?;tagged(&y,p,&r)}).await.map_err(bad)?.map_err(bad)?;
    save_or_download(bytes,suggested,"wav",true).await
}

pub async fn premaster(State(root):State<std::path::PathBuf>,Path(sid):Path<String>,Json(r):Json<Request>)->R<Response> {
    let dir=sess(&root,&sid)?;validate(&r).map_err(bad)?;let title=safe_name(&r.title);
    let bytes=tokio::task::spawn_blocking(move||->Result<Vec<u8>,String>{let p=&r.profiles[0];let y=request_mix(&dir,&r,p.sample_rate)?;if y.iter().any(|v|v.abs()>1.) {return Err("Mix exceeds 0 dBFS. Lower stem levels before PCM export.".into())}tagged(&y,&Profile{bits:24,..p.clone()},&r)}).await.map_err(bad)?.map_err(bad)?;
    save_or_download(bytes,format!("{title} - edited mix.wav"),"wav",true).await
}

fn default_match_level()->bool{true}
#[derive(Clone,Deserialize,Serialize)]
pub struct SideTrack {pub stem:String,pub side:String,pub gap:f64,
    #[serde(default="default_match_level")] pub match_level:bool,
    #[serde(default)] pub offset_db:f64}
#[derive(Clone,Deserialize,Serialize)]
pub struct SideRequest {pub title:String,pub sample_rate:u32,pub bits:u16,pub tracks:Vec<SideTrack>,
    #[serde(default)] pub level_matching:Option<crate::album::Settings>}
pub(crate) fn validate_sides(r:&SideRequest)->Result<(),String>{
    if r.title.trim().is_empty()||r.title.len()>200||![44100,48000,88200,96000].contains(&r.sample_rate)||![16,24].contains(&r.bits)||r.tracks.is_empty()||r.tracks.len()>100 {return Err("Ogiltiga albuminställningar".into())}
    let mut seen=HashSet::new();for t in &r.tracks {if !["A","B"].contains(&t.side.as_str())||!t.gap.is_finite()||!(0. ..=30.).contains(&t.gap)||!t.offset_db.is_finite()||!(-12. ..=12.).contains(&t.offset_db)||fname(&t.stem)!=t.stem||!crate::is_audio(&t.stem)||!seen.insert(&t.stem){return Err("Ogiltig låt, sida, nivå eller mellanrum".into())}}
    Ok(())
}
fn assemble_side(dir:&FsPath,r:&SideRequest,side:&str,plan:&crate::album::Plan)->Result<(Vec<f32>,Vec<Value>,String),String>{
    let mut y=Vec::new();let mut songs=Vec::new();let mut cue=String::new();
    for (i,t) in r.tracks.iter().enumerate().filter(|(_,t)|t.side==side) {
        if !y.is_empty(){y.resize(y.len()+(t.gap*r.sample_rate as f64).round() as usize*2,0.)}
        let start=y.len() as f64/2./r.sample_rate as f64;let (audio,rate,ch)=decode(&dir.join(&t.stem))?;let mut audio=stereo_resample(&audio,rate,ch,r.sample_rate)?;
        if y.len()+audio.len()>r.sample_rate as usize*60*60*2{return Err("En sida får vara högst 60 minuter".into())}
        let g=gain(plan.tracks[i].gain_db) as f32;for sample in &mut audio{*sample*=g}
        cue.push_str(&format!("{:02}:{:06.3}\t{}\n",(start/60.).floor() as u32,start%60.,t.stem));songs.push(json!({"file":t.stem,"start_seconds":start,"duration_seconds":audio.len() as f64/2./r.sample_rate as f64,"gain_db":plan.tracks[i].gain_db}));y.extend(audio);
    }
    Ok((y,songs,cue))
}
fn side_zip(dir:&FsPath,r:&SideRequest)->Result<Vec<u8>,String> {
    let plan=crate::album::plan(dir,r)?;
    // Check both assembled sides first. Any extra join headroom is applied to BOTH sides.
    let mut common=0f64;
    for side in ["A","B"] {let (y,_,_)=assemble_side(dir,r,side,&plan)?;if !y.is_empty(){common=common.min(plan.ceiling_db-0.15-measure(&y,r.sample_rate)?.true_peak_db.unwrap_or(-200.))}}
    let mut z=zip::ZipWriter::new(Cursor::new(Vec::new()));let mut sides=Vec::new();
    for side in ["A","B"] {
        let (mut y,songs,cue)=assemble_side(dir,r,side,&plan)?;if y.is_empty(){continue}
        for sample in &mut y {*sample*=gain(common) as f32}quantize(&mut y,r.bits);let output=measure(&y,r.sample_rate)?;
        if output.true_peak_db.is_some_and(|v|v>plan.ceiling_db){return Err("Albumets toppgräns överskreds".into())}
        zip_entry(&mut z,&format!("{} - Side {side}.wav",safe_name(&r.title)),&wav(&y,r.sample_rate,r.bits,false)?)?;
        zip_entry(&mut z,&format!("Side {side} - tracklist.txt"),cue.as_bytes())?;
        sides.push(json!({"side":side,"tracks":songs,"metrics":output,"safety_attenuation_db":-common}));
    }
    zip_entry(&mut z,"Side-report.json",&serde_json::to_vec_pretty(&json!({"sides":sides,"sample_rate":r.sample_rate,"bits":r.bits,"album_level_plan":plan,"request":r})).map_err(|e|e.to_string())?)?;
    z.finish().map(|z|z.into_inner()).map_err(|e|e.to_string())
}

pub async fn sides(State(root):State<std::path::PathBuf>,Path(sid):Path<String>,Json(r):Json<SideRequest>)->R<Response> {
    let dir=sess(&root,&sid)?;let name=format!("{} - sides.zip",safe_name(&r.title));
    let bytes=tokio::task::spawn_blocking(move||side_zip(&dir,&r)).await.map_err(bad)?.map_err(bad)?;save_or_download(bytes,name,"zip",true).await
}

#[cfg(test)]
mod tests {
    use super::*;
    fn profile(id:&str)->Profile {Profile{id:id.into(),sample_rate:44100,bits:if id=="CD"{16}else{24},target_lufs:Some(-14.),ceiling_db:-1.,bass_db:0.,mid_db:0.,treble_db:0.,compression:15.,mono_bass_hz:0.,highpass_hz:0.,limiter:true}}
    fn sine(sr:u32,seconds:f64,frequency:f64,amplitude:f64)->Vec<f32>{(0..(sr as f64*seconds) as usize).flat_map(|i|{let v=(amplitude*(2.*PI*frequency*i as f64/sr as f64).sin()) as f32;[v,v]}).collect()}
    #[test]fn loudness_reference_and_silence(){let m=measure(&sine(48000,5.,1000.,0.1),48000).unwrap();assert!((m.lufs.unwrap()+20.).abs()<0.4);assert!((m.true_peak_db.unwrap()+20.).abs()<0.1);assert!(measure(&vec![0.;48000],48000).unwrap().lufs.is_none());}
    #[test]fn profiles_reach_target_and_respect_true_peak(){let source=sine(44100,4.,1000.,0.05);for id in ["Spotify","CD","Vinyl","Kassett"]{let p=profile(id);let(y,r)=process(&source,&p).unwrap();assert_eq!(y.len(),source.len());assert!((r.output.lufs.unwrap()+14.).abs()<0.3);assert!(r.output.true_peak_db.unwrap()<=p.ceiling_db);let data=wav(&y,p.sample_rate,p.bits,false).unwrap();let reader=hound::WavReader::new(Cursor::new(data)).unwrap();assert_eq!(reader.spec().bits_per_sample,p.bits);assert_eq!(reader.spec().sample_rate,p.sample_rate);}}
    #[test]fn limiter_handles_impulses_and_high_frequency(){let mut y=sine(44100,3.,18000.,1.4);y[1000]=5.;y[1001]=-4.;let mut p=profile("Spotify");p.target_lufs=None;p.compression=0.;let(_,r)=process(&y,&p).unwrap();assert!(r.output.true_peak_db.unwrap()<=-1.);assert!(r.limiter_reduction_db>0.);}
    #[test]fn resampler_keeps_duration_and_rejects_aliases(){let source=sine(48000,1.,1000.,0.5);let out=stereo_resample(&source,48000,2,44100).unwrap();assert_eq!(out.len(),88200);assert!((measure(&source,48000).unwrap().lufs.unwrap()-measure(&out,44100).unwrap().lufs.unwrap()).abs()<0.1);let high=stereo_resample(&sine(96000,1.,30000.,0.5),96000,2,44100).unwrap();let rms=(high.iter().skip(1000).take(80000).map(|v|(*v as f64).powi(2)).sum::<f64>()/80000.).sqrt();assert!(rms<0.01);}
    #[test]fn mono_bass_reduces_antiphase_and_compressor_preserves_stereo(){let mut y=sine(44100,2.,40.,0.5);for f in y.chunks_exact_mut(2){f[1]=-f[0]}let before=measure(&y,44100).unwrap();let mut p=profile("Vinyl");p.mono_bass_hz=150.;tone(&mut y,&p);let power=y.iter().map(|v|(*v as f64).powi(2)).sum::<f64>();let original=sine(44100,2.,40.,0.5).iter().map(|v|(*v as f64).powi(2)).sum::<f64>();assert!(power/original<0.1);assert!(before.bass_side_ratio>0.9);let mut stereo=sine(44100,2.,1000.,0.8);compress(&mut stereo,&p);assert!(stereo.chunks_exact(2).all(|f|f[0]==f[1]));}
    #[test]fn batch_and_sides_keep_names_edits_and_reports(){let dir=std::env::temp_dir().join(format!("master-test-{}",std::process::id()));std::fs::create_dir_all(&dir).unwrap();let audio=sine(44100,2.,1000.,0.1);std::fs::write(dir.join("Vocals.wav"),wav(&audio,44100,24,true).unwrap()).unwrap();let request=Request{mix_edits:vec![],title:"Låt 1".into(),artist:"Chaoswrought".into(),album:"Album".into(),tracks:vec![MixTrack{stem:"Vocals.wav".into(),gain_db:0.,mute:false,solo:false,edits:vec![Edit{kind:"mute".into(),start:0.5,end:1.,fade:0.01,..Default::default()}]}],profiles:vec![profile("Spotify"),profile("CD")]};let zip=render_batch(&dir,&request).unwrap();let mut z=zip::ZipArchive::new(Cursor::new(zip)).unwrap();assert!(z.by_name("Mastering-report.json").is_ok());let mut file=z.by_name("Låt 1 - mastered - CD.wav").unwrap();let mut bytes=Vec::new();std::io::Read::read_to_end(&mut file,&mut bytes).unwrap();let mut reader=hound::WavReader::new(Cursor::new(bytes)).unwrap();let samples=reader.samples::<i16>().map(Result::unwrap).collect::<Vec<_>>();assert!(samples[44100..60000].iter().all(|s|*s==0));let sides=SideRequest{level_matching:None,title:"Album".into(),sample_rate:44100,bits:16,tracks:vec![SideTrack{match_level:true,offset_db:0.,stem:"Vocals.wav".into(),side:"A".into(),gap:0.}]};let bytes=side_zip(&dir,&sides).unwrap();let mut z=zip::ZipArchive::new(Cursor::new(bytes)).unwrap();assert!(z.by_name("Album - Side A.wav").is_ok());assert!(z.by_name("Side A - tracklist.txt").is_ok());std::fs::remove_dir_all(dir).unwrap();}
    #[test]
    fn panning_moves_both_channels_and_preserves_outside_samples() {
        let original:Vec<f32>=(0..1000).flat_map(|_|[0.2,0.3]).collect();
        let edit=Edit{kind:"pan".into(),start:0.2,end:0.8,fade:0.01,pan_from:-1.,pan_to:-1.,pan_cycles:0};
        let (left,ch)=render_edits(original.clone(),1000,2,&[edit.clone()]).unwrap();assert_eq!(ch,2);assert_eq!(&left[..400],&original[..400]);assert_eq!(&left[1600..],&original[1600..]);assert_eq!(left[1001],0.);assert!((left[1000]-0.5).abs()<1e-6);assert_eq!(left[400],original[400]);assert_eq!(left[401],original[401]);
        let center=Edit{pan_from:0.,pan_to:0.,..edit.clone()};let (reset,_)=render_edits(original.clone(),1000,2,&[edit.clone(),center]).unwrap();assert_eq!(reset,original);
        let partial=Edit{start:0.4,end:0.6,pan_from:0.,pan_to:0.,..edit.clone()};let (blended,_)=render_edits(original.clone(),1000,2,&[edit.clone(),partial]).unwrap();assert_eq!(blended[801],0.);assert!((blended[1001]-0.3).abs()<1e-6);
        let sweep=Edit{pan_to:1.,..edit};let (moving,_)=render_edits(original,1000,2,&[sweep]).unwrap();assert!(moving[440]>moving[441]);assert!(moving[1559]>moving[1558]);
    }
    #[test]
    fn mono_pan_and_periodic_motion_preserve_length_and_energy() {
        let original=vec![0.2;1000];let edit=Edit{kind:"pan".into(),start:0.2,end:0.8,fade:0.,pan_from:-1.,pan_to:1.,pan_cycles:2};
        let (y,ch)=render_edits(original,1000,1,&[edit]).unwrap();assert_eq!(ch,2);assert_eq!(y.len(),2000);assert_eq!(y[400+1],0.);assert!(y[700+1]>y[700]);
        for f in y.chunks_exact(2) {assert!(((f[0]*f[0]+f[1]*f[1])-0.08).abs()<1e-6)}
        let invalid=Edit{kind:"pan".into(),start:0.,end:1.,pan_to:2.,..Default::default()};assert!(render_edits(vec![0.;2000],1000,2,&[invalid]).is_err());
    }
    #[test]
    fn pan_is_isolated_to_its_stem_and_survives_master_export() {
        let dir=std::env::temp_dir().join(format!("pan-test-{}",std::process::id()));std::fs::create_dir_all(&dir).unwrap();
        let source:Vec<f32>=(0..44100).flat_map(|_|[0.1,0.2]).collect();for name in ["Synth.wav","Vocals.wav"] {std::fs::write(dir.join(name),wav(&source,44100,24,true).unwrap()).unwrap();}
        let track=|stem:&str,edits|MixTrack{stem:stem.into(),gain_db:0.,mute:false,solo:false,edits};let edit=Edit{kind:"pan".into(),start:0.2,end:0.8,fade:0.01,pan_from:-1.,pan_to:-1.,pan_cycles:0};
        let y=mix(&dir,&[track("Synth.wav",vec![edit.clone()]),track("Vocals.wav",vec![])],44100).unwrap();assert!((y[44100]-0.4).abs()<1e-6);assert!((y[44101]-0.2).abs()<1e-6);
        let r=Request{mix_edits:vec![],title:"Pan".into(),artist:"".into(),album:"".into(),tracks:vec![track("Synth.wav",vec![edit])],profiles:vec![profile("CD")]};let archive=render_batch(&dir,&r).unwrap();let mut z=zip::ZipArchive::new(Cursor::new(archive)).unwrap();let mut bytes=Vec::new();std::io::Read::read_to_end(&mut z.by_name("Pan - mastered - CD.wav").unwrap(),&mut bytes).unwrap();let mut reader=hound::WavReader::new(Cursor::new(bytes)).unwrap();let samples=reader.samples::<i16>().map(Result::unwrap).collect::<Vec<_>>();assert_eq!(samples[44101],0);assert_ne!(samples[44100],0);std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn mixing_honors_gain_mute_solo_and_longest_duration() {
        let dir=std::env::temp_dir().join(format!("mix-test-{}",std::process::id()));std::fs::create_dir_all(&dir).unwrap();
        let short=sine(44100,1.,1000.,0.1);let long=sine(48000,2.,2000.,0.2);
        std::fs::write(dir.join("Short.wav"),wav(&short,44100,24,true).unwrap()).unwrap();std::fs::write(dir.join("Long.wav"),wav(&long,48000,24,true).unwrap()).unwrap();
        let mut tracks=vec![MixTrack{stem:"Short.wav".into(),gain_db:-6.,mute:false,solo:true,edits:vec![]},MixTrack{stem:"Long.wav".into(),gain_db:0.,mute:false,solo:false,edits:vec![]}];
        let mixed=mix(&dir,&tracks,44100).unwrap();assert_eq!(mixed.len(),44100*4);assert!(mixed[short.len()..].iter().all(|s|*s==0.));assert!((mixed[1000] as f64-short[1000] as f64*gain(-6.)).abs()<1e-7);
        tracks[0].mute=true;tracks[1].mute=true;let silent=mix(&dir,&tracks,44100).unwrap();assert!(silent.iter().all(|s|*s==0.));
        tracks[1].stem="../bad.wav".into();assert!(mix(&dir,&tracks,44100).is_err());std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn dither_is_reproducible_and_preserves_exact_silence() {
        let mut a=vec![0.00001f32;2000];a[0]=0.;let mut b=a.clone();quantize(&mut a,16);quantize(&mut b,16);assert_eq!(a,b);assert_eq!(a[0],0.);assert!(a[1..].iter().any(|s|*s==0.));assert!(a[1..].iter().any(|s|*s>0.));
    }
    #[test]
    fn side_order_gaps_and_both_sides_roundtrip() {
        let dir=std::env::temp_dir().join(format!("side-test-{}",std::process::id()));std::fs::create_dir_all(&dir).unwrap();
        for name in ["One.wav","Two.wav","Three.wav"]{std::fs::write(dir.join(name),wav(&sine(44100,1.,1000.,0.1),44100,24,true).unwrap()).unwrap();}
        let r=SideRequest{level_matching:None,title:"Album".into(),sample_rate:44100,bits:16,tracks:vec![SideTrack{match_level:true,offset_db:0.,stem:"Two.wav".into(),side:"A".into(),gap:2.},SideTrack{match_level:true,offset_db:0.,stem:"One.wav".into(),side:"A".into(),gap:0.5},SideTrack{match_level:true,offset_db:0.,stem:"Three.wav".into(),side:"B".into(),gap:3.}]};
        let bytes=side_zip(&dir,&r).unwrap();let mut archive=zip::ZipArchive::new(Cursor::new(bytes)).unwrap();let mut bytes=Vec::new();std::io::Read::read_to_end(&mut archive.by_name("Side-report.json").unwrap(),&mut bytes).unwrap();let report:Value=serde_json::from_slice(&bytes).unwrap();assert_eq!(report["sides"][0]["tracks"][0]["file"],"Two.wav");assert_eq!(report["sides"][0]["tracks"][1]["start_seconds"],1.5);assert_eq!(report["sides"][1]["tracks"][0]["start_seconds"],0.0);assert_eq!(report["sides"][0]["metrics"]["duration"],2.5);std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]fn invalid_settings_rejected(){let mut p=profile("CD");p.bits=24;assert!(p.validate().is_err());p=profile("Vinyl");p.ceiling_db=f64::NAN;assert!(p.validate().is_err());assert_eq!(safe_name("../bad/name"),"_bad_name");}
}

#[cfg(test)] mod mix_edit_tests {
    use super::*;
    #[test] fn cut_keep_and_order_preserve_stereo_and_exact_duration(){
        let audio:Vec<f32>=(0..1000).flat_map(|i|[i as f32,-(i as f32)]).collect();
        let cut=MixEdit{kind:"cut".into(),start:0.2,end:0.4};
        let y=edit_mix(audio.clone(),1000,&[cut]).unwrap();assert_eq!(y.len(),1600);assert_eq!(y[600],500.);assert_eq!(y[601],-500.);assert_eq!(y[400],0.);
        let y=edit_mix(y,1000,&[MixEdit{kind:"keep".into(),start:0.1,end:0.6}]).unwrap();assert_eq!(y.len(),1000);assert_eq!(y[400],500.);assert_eq!(y[0],0.);assert_eq!(*y.last().unwrap(),0.);
        for (a,b) in [(0.,1.),(0.9,1.1),(f64::NAN,0.5)] {assert!(edit_mix(audio.clone(),1000,&[MixEdit{kind:"cut".into(),start:a,end:b}]).is_err())}
    }
}
