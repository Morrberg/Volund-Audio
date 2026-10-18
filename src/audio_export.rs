//! Audio encoders used by selected-track export.
use std::io::Cursor;

pub fn extension(format: &str) -> Result<&'static str, String> {
    match format {
        "wav" => Ok("wav"),
        "flac" => Ok("flac"),
        "mp3" => Ok("mp3"),
        "m4a" => Ok("m4a"),
        _ => Err("Välj WAV, FLAC, MP3 eller AAC (M4A)".into()),
    }
}

pub fn content_type(format: &str) -> Result<&'static str, String> {
    match format {
        "wav" => Ok("audio/wav"),
        "flac" => Ok("audio/flac"),
        "mp3" => Ok("audio/mpeg"),
        "m4a" => Ok("audio/mp4"),
        _ => Err("Välj WAV, FLAC, MP3 eller AAC (M4A)".into()),
    }
}

pub fn encode(
    samples: &[f32],
    sample_rate: u32,
    channels: usize,
    format: &str,
) -> Result<Vec<u8>, String> {
    if channels == 0 || samples.len() % channels != 0 || samples.iter().any(|s| !s.is_finite()) {
        return Err("Ljudet innehåller ogiltiga samplingar".into());
    }
    match format {
        "wav" => encode_wav(samples, sample_rate, channels),
        "flac" => encode_flac(samples, sample_rate, channels),
        "mp3" => encode_mp3(samples, sample_rate, channels),
        "m4a" => encode_m4a(samples, sample_rate, channels),
        _ => Err("Välj WAV, FLAC, MP3 eller AAC (M4A)".into()),
    }
}

fn encode_wav(samples: &[f32], sample_rate: u32, channels: usize) -> Result<Vec<u8>, String> {
    let spec = hound::WavSpec {
        channels: channels as u16,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut out = Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(&mut out, spec).map_err(|e| e.to_string())?;
        for &sample in samples {
            writer.write_sample(sample).map_err(|e| e.to_string())?;
        }
        writer.finalize().map_err(|e| e.to_string())?;
    }
    Ok(out.into_inner())
}

fn pcm24(samples: &[f32]) -> Result<Vec<i32>, String> {
    if samples.iter().any(|s| s.abs() > 1.0) {
        return Err(
            "Ljudet överstiger 0 dBFS. Sänk nivån innan export till FLAC, MP3 eller M4A.".into(),
        );
    }
    Ok(samples
        .iter()
        .map(|s| (s.clamp(-1.0, 1.0) * 8_388_607.0).round() as i32)
        .collect())
}

fn encode_flac(samples: &[f32], sample_rate: u32, channels: usize) -> Result<Vec<u8>, String> {
    if !(1..=8).contains(&channels) {
        return Err("FLAC-export stöder 1–8 ljudkanaler".into());
    }
    let pcm = pcm24(samples)?;
    use flacenc::{component::BitRepr, error::Verify};
    let config = flacenc::config::Encoder::default()
        .into_verified()
        .map_err(|_| "Ogiltig FLAC-konfiguration")?;
    let source = flacenc::source::MemSource::from_samples(&pcm, channels, 24, sample_rate as usize);
    let stream = flacenc::encode_with_fixed_block_size(&config, source, config.block_size)
        .map_err(|e| e.to_string())?;
    let mut sink = flacenc::bitsink::ByteSink::new();
    stream.write(&mut sink).map_err(|e| e.to_string())?;
    Ok(sink.as_slice().to_vec())
}

fn encode_mp3(samples: &[f32], sample_rate: u32, channels: usize) -> Result<Vec<u8>, String> {
    if !(1..=2).contains(&channels) {
        return Err("MP3-export stöder mono eller stereo".into());
    }
    let pcm = pcm24(samples)?
        .into_iter()
        .map(|s| s as f32 / 8_388_607.0)
        .collect::<Vec<_>>();
    let resampled = crate::mastering::stereo_resample(&pcm, sample_rate, channels, 48_000)?;
    let mut encoder = rusty_mp3::Mp3Encoder::new(rusty_mp3::Mp3EncoderConfig {
        bitrate_kbps: 320,
        vbr_quality: None,
    });
    encoder
        .push_pcm_f32(&resampled, 2, 48_000)
        .map_err(|e| e.to_string())?;
    encoder.finish();
    let mut out = Vec::new();
    while let Ok(packet) = encoder.next_packet() {
        out.extend_from_slice(&packet);
    }
    if out.is_empty() {
        return Err("MP3-kodningen gav ingen ljuddata".into());
    }
    Ok(out)
}

fn encode_m4a(samples: &[f32], sample_rate: u32, channels: usize) -> Result<Vec<u8>, String> {
    if !(1..=2).contains(&channels) {
        return Err("AAC-export stöder mono eller stereo".into());
    }
    let pcm24 = pcm24(samples)?;
    let pcm = pcm24
        .into_iter()
        .map(|s| s as f32 / 8_388_607.0)
        .collect::<Vec<_>>();
    let resampled = crate::mastering::stereo_resample(&pcm, sample_rate, channels, 48_000)?;
    let mut encoder = rusty_aac::AacEncoder::new(rusty_aac::AacEncoderConfig {
        bitrate_bps: 256_000,
        ..Default::default()
    });
    encoder
        .push_pcm(&resampled, 2, 48_000)
        .map_err(|e| e.to_string())?;
    encoder.finish();

    use bytes::Bytes;
    use mp4::{
        AacConfig, AudioObjectType, ChannelConfig, MediaConfig, Mp4Config, Mp4Sample, Mp4Writer,
        SampleFreqIndex, TrackConfig, TrackType,
    };
    let config = Mp4Config {
        major_brand: "M4A ".parse().map_err(|e: mp4::Error| e.to_string())?,
        minor_version: 0,
        compatible_brands: vec![
            "M4A ".parse().map_err(|e: mp4::Error| e.to_string())?,
            "mp42".parse().map_err(|e: mp4::Error| e.to_string())?,
        ],
        timescale: 48_000,
    };
    let mut writer =
        Mp4Writer::write_start(Cursor::new(Vec::new()), &config).map_err(|e| e.to_string())?;
    writer
        .add_track(&TrackConfig {
            track_type: TrackType::Audio,
            timescale: 48_000,
            language: "und".into(),
            media_conf: MediaConfig::AacConfig(AacConfig {
                bitrate: 256_000,
                profile: AudioObjectType::AacLowComplexity,
                freq_index: SampleFreqIndex::Freq48000,
                chan_conf: ChannelConfig::Stereo,
            }),
        })
        .map_err(|e| e.to_string())?;
    while let Ok(packet) = encoder.next_packet() {
        writer
            .write_sample(
                1,
                &Mp4Sample {
                    start_time: packet.pts as u64,
                    duration: packet.duration,
                    rendering_offset: 0,
                    is_sync: true,
                    bytes: Bytes::from(packet.data),
                },
            )
            .map_err(|e| e.to_string())?;
    }
    writer.write_end().map_err(|e| e.to_string())?;
    Ok(writer.into_writer().into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<f32> {
        (0..4800)
            .flat_map(|i| {
                let v = 0.2 * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 48_000.0).sin();
                [v, v]
            })
            .collect()
    }

    #[test]
    fn writes_valid_container_headers_for_each_export_format() {
        let pcm = sample();
        let wav = encode(&pcm, 48_000, 2, "wav").unwrap();
        let flac = encode(&pcm, 48_000, 2, "flac").unwrap();
        let mp3 = encode(&pcm, 48_000, 2, "mp3").unwrap();
        let m4a = encode(&pcm, 48_000, 2, "m4a").unwrap();
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&flac[0..4], b"fLaC");
        assert!(mp3.len() > 4 && mp3[0] == 0xff && mp3[1] & 0xe0 == 0xe0);
        assert_eq!(&m4a[4..8], b"ftyp");
    }

    #[test]
    fn integer_encoders_reject_samples_above_full_scale() {
        assert!(encode(&[1.1, 0.0], 48_000, 2, "flac").is_err());
        assert!(encode(&[1.1, 0.0], 48_000, 2, "mp3").is_err());
        assert!(encode(&[1.1, 0.0], 48_000, 2, "m4a").is_err());
        assert!(encode(&[0.0], 48_000, 2, "wav").is_err());
    }
}
