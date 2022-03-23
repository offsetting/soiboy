use std::fs::File;
use std::io::BufReader;
use std::mem::size_of_val;
use std::path::PathBuf;

use binrw::{BinRead, BinReaderExt, BinResult};

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
  uncached: Vec<i32>,

  #[br(count = cached_amount)]
  cached: Vec<i32>,
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

fn extract(archive: PathBuf, destination: PathBuf) -> BinResult<()> {
  let mut file = BufReader::new(File::open(archive)?);

  let mut sections: Vec<Section> = Vec::new();

  // read sections until an error happens
  loop {
    match file.read_be() {
      Ok(section) => sections.push(section),
      Err(_) => break
    };
  }

  println!("{:#?}", sections);

  Ok(())
}

fn main() {
  extract(
    "./data/VehicleInfo.x360.toc".parse().unwrap(),
    "./data/dist".parse().unwrap(),
  )
    .unwrap()
}

fn archive(input: PathBuf, archive: PathBuf) {}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use crate::extract;

  #[test]
  fn it_works() {
    extract(
      "./data/VehicleInfo.x360.toc".parse().unwrap(),
      "./data/dist".parse().unwrap(),
    )
      .unwrap()
  }
}
