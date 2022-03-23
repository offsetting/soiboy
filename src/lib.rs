use std::fs::{create_dir_all, File};
use std::io::{BufReader, Read};
use std::io;
use std::io::prelude::*;
use std::mem::size_of_val;
use std::path::{Path, PathBuf};

use binrw::{BinRead, BinReaderExt, BinResult};
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

#[derive(BinRead, Debug)]
struct Bounding {
  min_x: f32,
  max_x: f32,

  min_y: f32,
  max_y: f32,

  min_z: f32,
  max_z: f32,
}

#[derive(BinRead, Debug)]
struct MemoryEntry {
  offset: i32,
  size: i32,
}

#[derive(BinRead, Debug)]
#[br(repr = i32)]
enum ComponentType {
  RenderableModel,
  Texture,
  CollisionModel,
  UserData,
  MotionPack,
  CollisionGrid,
}

#[derive(BinRead, Debug)]
struct ZlibHeader {
  uncached_total_size: i32,
  cached_total_size: i32,

  uncached_amount: i32,
  cached_amount: i32,

  #[br(count = uncached_amount)]
  uncached_sizes: Vec<i32>,

  #[br(count = cached_amount)]
  cached_sizes: Vec<i32>,
}

#[derive(BinRead, Debug)]
struct SectionHeader {
  name: [char; 260],

  total_component_count: i32,
  uncached_component_count: i32,
  cached_component_count: i32,

  shared_section_offset: i32,
  uncached_page_offset: i32,
  cached_page_offset: i32,

  link_table: [i32; 8],

  bounding: Bounding,

  memory_entry: MemoryEntry,
  uncached_data_size: i32,
  cached_data_size: i32,

  zlib_header: ZlibHeader,
}

#[derive(BinRead, Debug)]
struct ComponentHeader {
  path: [char; 260],

  instance_id: i32,
  component_id: i32,
  memory_entry: MemoryEntry,
  component_type: ComponentType,
}

#[derive(BinRead, Debug)]
struct Section {
  header: SectionHeader,

  #[br(big, count = header.uncached_component_count)]
  uncached_components: Vec<ComponentHeader>,

  #[br(big, count = header.cached_component_count)]
  cached_components: Vec<ComponentHeader>,
}

fn extract(toc: PathBuf, str: PathBuf, destination: PathBuf) -> BinResult<()> {
  let mut toc_file = BufReader::new(File::open(toc)?);
  let mut str_file = BufReader::new(File::open(str)?);

  let mut sections: Vec<Section> = Vec::new();

  // read sections until an error happens
  loop {
    match toc_file.read_be() {
      Ok(section) => sections.push(section),
      Err(_) => break
    };
  }

  println!("Found {} sections", sections.len());

  println!("{:#?}", sections.iter().next().unwrap());

  // for section in sections {
  process_section(sections.iter().next().unwrap(), &mut str_file);
  // }

  Ok(())
}

fn process_section(section: &Section, str: &mut BufReader<File>) -> io::Result<()> {
  let zlib_header = &section.header.zlib_header;

  let uncached = decode_zlib_chunks(str, &zlib_header.uncached_sizes)?;
  decode_components(&uncached, &section.uncached_components)?;

  let cached = decode_zlib_chunks(str, &zlib_header.cached_sizes)?;
  decode_components(&cached, &section.cached_components)?;

  Ok(())
}

fn decode_zlib_chunks(str: &mut BufReader<File>, sizes: &[i32]) -> io::Result<Vec<u8>> {
  let mut whole_section = Vec::new();

  for size in sizes {
    // reading compressed chunk
    let mut buf = vec![0; size.to_owned() as usize];
    str.read_exact(&mut buf)?;

    // decompressing chunk and appending to merged vector
    let mut decoder = ZlibDecoder::new(&buf[..]);
    decoder.read_to_end(&mut whole_section)?;
  }

  Ok(whole_section)
}

fn decode_components(data: &[u8], components: &[ComponentHeader]) -> io::Result<()> {
  for component in components {
    let start = component.memory_entry.offset as usize;
    let end = start + component.memory_entry.size as usize;

    let mut comp_data = &data[start..end];

    println!("{}..{} {} - {} {}", start, end, data.len(), component.component_id, component.path.iter().collect::<String>());

    let out_path = PathBuf::from(format!("./data/out/{}", component.component_id));
    create_dir_all(out_path.parent().unwrap());

    let mut out = File::create(&out_path)?;
    io::copy(&mut comp_data, &mut out)?;
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use crate::extract;

  #[test]
  fn it_works() {
    extract(
      PathBuf::from("./data/VehicleInfo.x360.toc"),
      PathBuf::from("data/VehicleInfo.x360.str"),
      PathBuf::from("./data/dist"),
    )
      .unwrap()
  }
}
