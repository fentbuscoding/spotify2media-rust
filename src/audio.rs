use std::fs;
use std::io::Read;
use std::path::Path;
use anyhow::{Context, Result};
use lofty::{Accessor, AudioFile, Tag, TagType, TaggedFileExt};

pub fn set_mp3_tags(path: impl AsRef<Path>, title: &str, artist: &str, album: &str) -> Result<()> {
    let path = path.as_ref();
    let mut tagged_file = lofty::read_from_path(path)
        .with_context(|| format!("Failed to read tags from {:?}", path))?;

    if tagged_file.primary_tag_mut().is_none() {
        tagged_file.insert_tag(Tag::new(TagType::Id3v2));
    }
    if let Some(tag) = tagged_file.primary_tag_mut() {
        tag.set_title(title.to_string());
        tag.set_artist(artist.to_string());
        tag.set_album(album.to_string());
    }

    tagged_file.save_to_path(path)
        .with_context(|| format!("Failed to save tags to {:?}", path))?;
    Ok(())
}

pub fn set_m4a_tags(path: impl AsRef<Path>, title: &str, artist: &str, album: &str) -> Result<()> {
    let path = path.as_ref();
    let mut tagged_file = lofty::read_from_path(path)
        .with_context(|| format!("Failed to read tags from {:?}", path))?;

    if tagged_file.primary_tag_mut().is_none() {
        tagged_file.insert_tag(Tag::new(TagType::Mp4Ilst));
    }
    if let Some(tag) = tagged_file.primary_tag_mut() {
        tag.set_title(title.to_string());
        tag.set_artist(artist.to_string());
        tag.set_album(album.to_string());
    }

    tagged_file.save_to_path(path)
        .with_context(|| format!("Failed to save tags to {:?}", path))?;
    Ok(())
}

pub fn is_valid_mp3(path: &Path) -> bool {
    if let Ok(mut file) = fs::File::open(path) {
        let mut header = [0u8; 3];
        if file.read_exact(&mut header).is_ok() {
            return &header == b"ID3" || &header == b"TAG";
        }
    }
    false
}