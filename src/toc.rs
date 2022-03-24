use std::fs::File;
use std::io::{BufReader, Seek};
use std::path::PathBuf;

use binrw::{BinRead, BinReaderExt, BinResult};

#[derive(BinRead, Debug)]
pub(crate) struct Bounding {
  pub(crate) min_x: f32,
  pub(crate) max_x: f32,

  pub(crate) min_y: f32,
  pub(crate) max_y: f32,

  pub(crate) min_z: f32,
  pub(crate) max_z: f32,
}

#[derive(BinRead, Debug)]
pub(crate) struct MemoryEntry {
  pub(crate) offset: i32,
  pub(crate) size: i32,
}

#[derive(BinRead, PartialEq, Debug)]
#[br(repr = i32)]
pub(crate) enum ComponentType {
  RenderableModel,
  Texture,
  CollisionModel,
  UserData,
  MotionPack,
  CollisionGrid,
}

#[derive(BinRead, Debug)]
pub(crate) struct ZlibHeader {
  pub(crate) uncached_total_size: i32,
  pub(crate) cached_total_size: i32,

  pub(crate) uncached_amount: i32,
  pub(crate) cached_amount: i32,

  #[br(count = uncached_amount)]
  pub(crate) uncached_sizes: Vec<i32>,

  #[br(count = cached_amount)]
  pub(crate) cached_sizes: Vec<i32>,
}

#[derive(BinRead, Debug)]
pub(crate) struct SectionHeader {
  pub(crate) name: [char; 260],

  pub(crate) total_component_count: i32,
  pub(crate) uncached_component_count: i32,
  pub(crate) cached_component_count: i32,

  pub(crate) shared_section_offset: i32,
  pub(crate) uncached_page_offset: i32,
  pub(crate) cached_page_offset: i32,

  pub(crate) link_table: [i32; 8],

  pub(crate) bounding: Bounding,

  pub(crate) memory_entry: MemoryEntry,
  pub(crate) uncached_data_size: i32,
  pub(crate) cached_data_size: i32,

  pub(crate) zlib_header: ZlibHeader,
}

#[derive(BinRead, Debug)]
pub(crate) struct ComponentHeader {
  pub(crate) path: [char; 260],

  pub(crate) instance_id: i32,
  pub(crate) component_id: i32,
  pub(crate) memory_entry: MemoryEntry,
  pub(crate) component_type: ComponentType,
}

#[derive(BinRead, Debug)]
pub(crate) struct Section {
  pub(crate) header: SectionHeader,

  #[br(count = header.uncached_component_count)]
  pub(crate) uncached_components: Vec<ComponentHeader>,

  #[br(count = header.cached_component_count)]
  pub(crate) cached_components: Vec<ComponentHeader>,
}

pub(crate) fn read_toc(path: PathBuf) -> BinResult<Vec<Section>> {
  let mut file = File::open(path)?;

  let mut sections = Vec::new();
  let file_size = file.metadata()?.len();

  // read sections until the end of the file is reached
  while file.stream_position()? < file_size {
    let section = file.read_be()?;
    sections.push(section);
  }

  Ok(sections)
}
