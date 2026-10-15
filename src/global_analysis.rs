//! Time ranges refer to the edited mix, not the original stem timeline.
use ebur128::{EbuR128, Mode};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub fn analyze(y: &[f32], sr: u32) -> Result<Value, String> {
    if sr == 0 || y.len()%2 != 0 { return Err("Invalid stereo audio".into()); }
    let mut meter=EbuR128::new(2,sr,Mode::S|Mode::TRUE_PEAK).map_err(|e|e.to_string())?;
    let hop=(sr as usize/10).max(1);
    let duration=y.len() as f64/2./sr as f64;
    let mut peak=0f64;let mut peak_window=None;
    let mut maximum:Option<(f64,f64)>=None;let mut minimum:Option<(f64,f64)>=None;
    let mut series=Vec::new();let mut histogram=BTreeMap::<i32,usize>::new();
    let mut silent=0usize;let mut processed=0usize;
    for block in y.chunks(hop*2) {
        let start=processed as f64/sr as f64;
        processed+=block.len()/2;let end=processed as f64/sr as f64;
        meter.add_frames_f32(block).map_err(|e|e.to_string())?;
        let p=(0..2).filter_map(|c|meter.prev_true_peak(c).ok()).fold(0f64,f64::max);
        if p>peak {peak=p;peak_window=Some((start,end));}
        if processed>=sr as usize*3 {
            let value=meter.loudness_shortterm().ok().filter(|v|v.is_finite());
            if let Some(v)=value {
                if maximum.is_none_or(|(m,_)|v>m) {maximum=Some((v,end));}
                if minimum.is_none_or(|(m,_)|v<m) {minimum=Some((v,end));}
                *histogram.entry(v.floor() as i32).or_default()+=1;
            } else {silent+=1;}
            series.push(json!({"start_seconds":end-3.,"end_seconds":end,"lufs":value}));
        }
    }
    // Flush the true-peak interpolator; its final tail belongs to the last interval.
    meter.add_frames_f32(&[0.;128]).map_err(|e|e.to_string())?;
    let tail=(0..2).filter_map(|c|meter.prev_true_peak(c).ok()).fold(0f64,f64::max);
    if tail>peak {peak=tail;peak_window=Some(((duration-0.1).max(0.),duration));}
    let extremum=|point:Option<(f64,f64)>|point.map(|(v,end)|json!({"lufs":v,"start_seconds":end-3.,"end_seconds":end}));
    let dc:Vec<f64>=(0..2).map(|c|y.chunks_exact(2).map(|f|f[c] as f64).sum::<f64>()/(y.len()/2).max(1) as f64).collect();
    Ok(json!({"step_seconds":0.1,"short_term_window_seconds":3.,
        "true_peak_db":if peak>1e-15 {Some(20.*peak.log10())}else{None},
        "true_peak_window":peak_window.map(|(a,b)|json!({"start_seconds":a,"end_seconds":b})),
        "true_peak_timing_note":"100 ms detector interval; interpolation filter delay is not compensated",
        "short_term_max":extremum(maximum),"short_term_min":extremum(minimum),
        "silent_short_term_windows":silent,"dc_offset_per_channel":dc,
        "distribution":histogram.into_iter().map(|(bin,count)|json!({"from_lufs":bin,"to_lufs":bin+1,"windows":count})).collect::<Vec<_>>(),
        "short_term_series":series}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn silence_and_short_files_have_no_loudness_extrema() {
        for seconds in [1,4] {let r=analyze(&vec![0.;48000*2*seconds],48000).unwrap();assert!(r["short_term_max"].is_null());assert!(r["true_peak_window"].is_null());}
    }
    #[test]
    fn finds_loud_section_and_peak_interval() {
        let sr=48000;let mut y=Vec::new();
        for i in 0..sr*9 {let a=if i<sr*4 {0.01}else{0.2};let v=a*(2.*std::f64::consts::PI*1000.*i as f64/sr as f64).sin();y.extend([v as f32,v as f32]);}
        y[48000*7*2]=0.9;
        let r=analyze(&y,sr).unwrap();assert!(r["short_term_max"]["lufs"].as_f64().unwrap()>r["short_term_min"]["lufs"].as_f64().unwrap()+10.);
        let start=r["true_peak_window"]["start_seconds"].as_f64().unwrap();assert!((start-7.).abs()<0.11);assert!(r["distribution"].as_array().unwrap().len()>1);
    }
}
