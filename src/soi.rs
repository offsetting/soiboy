use std::fs::File;
use std::path::PathBuf;

use binrw::{BinRead, BinReaderExt, BinResult};

#[derive(BinRead, PartialEq, Debug)]
#[br(repr = i32)]
pub(crate) enum StreamingMode {
  Unknown = -1,
  _1D,
  _2D,
  Manuel,
}

#[derive(BinRead, Debug)]
pub(crate) struct Header {
  pub(crate) version: i32,

  pub(crate) flags: i32,
  pub(crate) sections: i32,
  pub(crate) collision_models: i32,
  pub(crate) renderable_models: i32,
  pub(crate) motion_packs: i32,
  pub(crate) streaming_textures: i32,
  pub(crate) static_textures: i32,
  pub(crate) uncached_pages: i32,
  pub(crate) cached_pages: i32,

  pub(crate) motion_packs_offset: i32,
  pub(crate) renderable_models_offset: i32,
  pub(crate) collision_models_offset: i32,
  pub(crate) textures_offset: i32,
  pub(crate) collision_grids_offset: i32,

  pub(crate) streaming_mode: StreamingMode,
  pub(crate) reserved: [u8; 16],
}

#[derive(BinRead, Debug)]
pub(crate) struct ModelInfo {
  pub(crate) flags: i32,
  pub(crate) position: [f32; 4],
  pub(crate) look_vector: [f32; 4],
  pub(crate) up_vector: [f32; 4],
  pub(crate) is_animated: i32,
  pub(crate) section_id: i32,
  pub(crate) component_id: i32,

  pub(crate) name: [char; 260],

  pub(crate) zone: i32,
  pub(crate) parameter_count: i32,
}

#[derive(BinRead, Debug)]
pub(crate) struct StreamingTexture {
  pub(crate) model_info: ModelInfo,
  // might be something, currently only padding
  pub(crate) padding: u32,
  pub(crate) texture: [u8; 52],
}

#[derive(BinRead, Debug)]
pub(crate) struct Soi {
  pub(crate) header: Header,

  #[br(count = header.uncached_pages)]
  pub(crate) uncached_page_sizes: Vec<i32>,

  #[br(count = header.cached_pages)]
  pub(crate) cached_page_sizes: Vec<i32>,

  #[br(count = header.streaming_textures)]
  pub(crate) streaming_textures: Vec<StreamingTexture>,

  // #[br(count = header.static_textures)]
  // static_textures: Vec<StaticTexture>,

  // #[br(count = header.motion_packs)]
  // motion_packs: Vec<MotionPack>,
}

pub(crate) fn read_soi(path: PathBuf) -> BinResult<Soi> {
  File::open(path)?.read_be()
}
