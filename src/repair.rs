/// Reconstruct short, full-scale, same-polarity plateaus with a cubic Hermite spline.
/// The slope at each endpoint comes from its nearest intact neighbour.
/// Returns repaired plateau count. A linked gain creates sample-peak headroom.
pub fn declip(y: &mut [f32], ch: usize) -> usize {
    if ch == 0 { return 0; }
    let frames = y.len() / ch;
    let mut repaired = 0;
    for c in 0..ch {
        let mut i = 0;
        while i < frames {
            let value = y[i*ch+c];
            if !value.is_finite() || value.abs() < 0.9999 { i += 1; continue; }
            let a = i;
            let sign = value.signum();
            while i < frames && y[i*ch+c].is_finite() && y[i*ch+c].abs() >= 0.9999 && y[i*ch+c].signum() == sign { i += 1; }
            let b = i;
            let n = b-a;
            if n < 2 || n > 64 || a < 2 || b+1 >= frames { continue; }
            let l = y[(a-1)*ch+c] as f64;
            let r = y[b*ch+c] as f64;
            let m0 = l-y[(a-2)*ch+c] as f64;
            let m1 = y[(b+1)*ch+c] as f64-r;
            if sign as f64*m0 <= 0. || sign as f64*m1 >= 0. { continue; }
            let d = (n+1) as f64;
            let mut values = Vec::with_capacity(n);
            for k in 0..n {
                let t = (k+1) as f64/d;
                let t2=t*t; let t3=t2*t;
                values.push((2.*t3-3.*t2+1.)*l+(t3-2.*t2+t)*d*m0+(-2.*t3+3.*t2)*r+(t3-t2)*d*m1);
            }
            if values.iter().any(|v| !v.is_finite() || sign as f64*v < 0.9999 || v.abs() > 2.) { continue; }
            for (k,v) in values.iter().enumerate() { y[(a+k)*ch+c] = *v as f32; }
            repaired += 1;
        }
    }
    if repaired > 0 {
        let peak = y.iter().fold(1f32, |p,v| p.max(v.abs()));
        if peak > 1. { for v in y { *v *= 0.999/peak; } }
    }
    repaired
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repairs_both_polarities_and_leaves_intact_audio() {
        for sign in [-1.,1.] {
            let mut y:Vec<f32> = [0.5,0.8,0.95,1.,1.,0.95,0.8,0.5].iter().map(|v|v*sign).collect();
            assert_eq!(declip(&mut y,1),1);
            assert!(y.iter().all(|v|v.is_finite()&&v.abs()<=0.99901));
            assert!(y[3].abs()>y[2].abs());
        }
        let mut clean=vec![0.2,-0.3,0.5];let original=clean.clone();assert_eq!(declip(&mut clean,1),0);assert_eq!(clean,original);
    }
    #[test]
    fn skips_boundaries_and_long_damage() {
        let mut y=vec![1.;100];let original=y.clone();assert_eq!(declip(&mut y,1),0);assert_eq!(y,original);
    }
}

/// First-order normalized DC blocker at 2 Hz, separate state per channel.
/// State starts at zero; DC steps settle rather than receiving an instantaneous correction.
pub fn dc_block(y: &mut [f32], sr: u32, ch: usize) {
    if sr==0 || ch==0 { return; }
    let r=(-2.*std::f64::consts::PI*2./sr as f64).exp();
    let scale=(1.+r)*0.5;
    let mut x1=vec![0f64;ch];let mut y1=vec![0f64;ch];
    for frame in y.chunks_exact_mut(ch) {for c in 0..ch {
        let x=frame[c] as f64;let out=scale*(x-x1[c])+r*y1[c];
        x1[c]=x;y1[c]=out;frame[c]=out as f32;
    }}
}

#[cfg(test)]
mod dc_tests {
    use super::*;
    #[test]
    fn removes_dc_without_channel_crosstalk_at_supported_rates() {
        for sr in [44100,48000,88200,96000] {
            let mut y=vec![0.;sr as usize*2];for f in y.chunks_exact_mut(2){f[0]=0.2;f[1]=-0.1;}
            dc_block(&mut y,sr,2);assert!(y[y.len()-2].abs()<0.00001);assert!(y[y.len()-1].abs()<0.00001);
        }
    }
    #[test]
    fn preserves_audible_tone_after_settling() {
        let sr=48000;let mut y:Vec<f32>=(0..sr*2).map(|i|(2.*std::f64::consts::PI*1000.*i as f64/sr as f64).sin() as f32*0.2+0.1).collect();
        dc_block(&mut y,sr,1);let tail=&y[sr as usize..];let mean=tail.iter().map(|v|*v as f64).sum::<f64>()/sr as f64;let rms=(tail.iter().map(|v|(*v as f64).powi(2)).sum::<f64>()/sr as f64).sqrt();assert!(mean.abs()<0.00001);assert!((rms-0.2/2f64.sqrt()).abs()<0.001);
    }
}
