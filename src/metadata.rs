//! Metadata is read from originals and written only to exported WAV copies.
use lofty::{config::WriteOptions, file::{AudioFile, TaggedFileExt}, tag::{ItemKey, ItemValue, Tag, TagExt, TagItem, TagType}};
use serde::{Deserialize, Serialize};
use std::{io::Cursor, path::Path};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Field { pub key: String, pub value: String }
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Changes { pub fields: Vec<Field>, #[serde(default)] pub clear: bool }

fn key(name: &str) -> Option<ItemKey> {
    ItemKey::VARIANTS.iter().copied().find(|k| format!("{k:?}") == name)
}
fn text(value: &ItemValue) -> Option<&str> { value.text().or_else(|| value.locator()) }

pub fn inspect(path: &Path) -> Result<serde_json::Value, String> {
    let file = lofty::read_from_path(path).map_err(|e| e.to_string())?;
    let mut fields = Vec::new();
    let mut pictures = 0; let mut binary = 0;
    for tag in file.tags() {
        pictures += tag.pictures().len();
        for item in tag.items() {
            if let Some(value) = text(item.value()) {
                fields.push(serde_json::json!({"key": format!("{:?}", item.key()), "value":value,
                    "editable": item.key().map_key(TagType::Id3v2).is_some(), "format":format!("{:?}", tag.tag_type())}));
            } else { binary += 1; }
        }
    }
    Ok(serde_json::json!({"fields":fields,"pictures":pictures,"binary":binary,
        "formats":file.tags().iter().map(|t|format!("{:?}",t.tag_type())).collect::<Vec<_>>(),
        "sampleRate":file.properties().sample_rate(),"channels":file.properties().channels()}))
}

pub fn apply(path: &Path, wav: Vec<u8>, changes: Option<&Changes>) -> Result<Vec<u8>, String> {
    if changes.is_some_and(|c| c.clear) { return write_tags(wav, &[], changes); }
    let original = lofty::read_from_path(path).map_err(|e| e.to_string())?;
    write_tags(wav, original.tags(), changes)
}

fn write_tags(wav: Vec<u8>, originals: &[Tag], changes: Option<&Changes>) -> Result<Vec<u8>, String> {
    let mut tag = Tag::new(TagType::Id3v2);
    let clear = changes.is_some_and(|c| c.clear);
    if let Some(changes) = changes {
        for field in &changes.fields {
            if field.value.is_empty() { continue; }
            let key = key(&field.key).ok_or_else(|| format!("Okänt metadatafält: {}", field.key))?;
            if key.map_key(TagType::Id3v2).is_none() { return Err(format!("Fältet {} kan inte skrivas till WAV", field.key)); }
            tag.push(TagItem::new(key, ItemValue::Text(field.value.clone())));
        }
    }
    for source in originals {
        for item in source.items() {
            if changes.is_none() || (!clear && text(item.value()).is_none()) {
                if item.key().map_key(TagType::Id3v2).is_some() { tag.push(item.clone()); }
            }
        }
        if !clear { for picture in source.pictures() { tag.push_picture(picture.clone()); } }
    }
    // hound's freshly rendered WAV has no descriptive tags; clearing needs no write.
    if tag.items().next().is_none() && tag.pictures().is_empty() { return Ok(wav); }
    let mut buffer = Cursor::new(wav);
    tag.save_to(&mut buffer, WriteOptions::default()).map_err(|e| e.to_string())?;
    Ok(buffer.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn wav() -> Vec<u8> {
        let mut cursor=Cursor::new(Vec::new());
        let spec=hound::WavSpec{channels:2,sample_rate:44100,bits_per_sample:32,sample_format:hound::SampleFormat::Float};
        { let mut writer=hound::WavWriter::new(&mut cursor,spec).unwrap();for sample in [0.25f32,-0.5,0.75,0.0]{writer.write_sample(sample).unwrap()}writer.finalize().unwrap(); }
        cursor.into_inner()
    }
    #[test]
    fn metadata_edit_clear_and_audio_roundtrip() {
        let original=wav();let changes=Changes{clear:false,fields:vec![Field{key:"TrackTitle".into(),value:"Sång – äöå".into()},Field{key:"TrackArtist".into(),value:"Chaoswrought".into()}]};
        let exported=write_tags(original.clone(),&[],Some(&changes)).unwrap();
        let mut cursor=Cursor::new(exported.clone());let file=lofty::file::TaggedFile::read_from(&mut cursor, lofty::config::ParseOptions::default()).unwrap();
        assert_eq!(file.tag(TagType::Id3v2).unwrap().get_string(ItemKey::TrackTitle),Some("Sång – äöå"));
        let mut reader=hound::WavReader::new(Cursor::new(exported)).unwrap();
        assert_eq!(reader.spec().channels,2);assert_eq!(reader.spec().sample_rate,44100);
        assert_eq!(reader.samples::<f32>().map(Result::unwrap).collect::<Vec<_>>(),vec![0.25,-0.5,0.75,0.0]);
        let cleared=write_tags(original.clone(),file.tags(),Some(&Changes{clear:true,fields:vec![]})).unwrap();
        assert_eq!(cleared,original);
        let preserved=write_tags(original,file.tags(),None).unwrap();
        let file=lofty::file::TaggedFile::read_from(&mut Cursor::new(preserved), lofty::config::ParseOptions::default()).unwrap();
        assert_eq!(file.tag(TagType::Id3v2).unwrap().get_string(ItemKey::TrackArtist),Some("Chaoswrought"));
    }
}
