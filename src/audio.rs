use std::path::Path;
use anyhow::Result;
use lofty::{Accessor, AudioFile, Tag, TagType, TaggedFileExt};

pub fn set_mp3_tags(path: &Path, title: &str, artist: &str, album: &str) -> Result<()> {
    let mut tagged_file = lofty::read_from_path(path)?;

    if tagged_file.primary_tag_mut().is_none() {
        tagged_file.insert_tag(Tag::new(TagType::Id3v2));
    }
    if let Some(tag) = tagged_file.primary_tag_mut() {
        tag.set_title(title.to_string());
        tag.set_artist(artist.to_string());
        tag.set_album(album.to_string());
    }

    tagged_file.save_to_path(path)?;
    Ok(())
}

pub fn set_m4a_tags(path: &Path, title: &str, artist: &str, album: &str) -> Result<()> {
    let mut tagged_file = lofty::read_from_path(path)?;

    if tagged_file.primary_tag_mut().is_none() {
        tagged_file.insert_tag(Tag::new(TagType::Mp4Ilst));
    }
    if let Some(tag) = tagged_file.primary_tag_mut() {
        tag.set_title(title.to_string());
        tag.set_artist(artist.to_string());
        tag.set_album(album.to_string());
    }

    tagged_file.save_to_path(path)?;
    Ok(())
}