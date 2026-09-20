use std::{fs::File, io::{self, Read}, path::Path};
use zip::ZipArchive;
pub fn read_json_entry(save: &Path, entry: &str) -> io::Result<String> { let file = File::open(save)?; let mut archive = ZipArchive::new(file).map_err(io::Error::other)?; let mut item = archive.by_name(entry).map_err(io::Error::other)?; let mut text = String::new(); item.read_to_string(&mut text)?; Ok(text) }
