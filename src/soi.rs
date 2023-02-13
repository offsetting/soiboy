use std::fs::File;
use std::path::Path;

use binrw::{BinRead, BinReaderExt, BinResult};

use crate::collision::*;
use crate::model::*;
use crate::motion::*;
use crate::utils::*;

#[derive(BinRead, PartialEq, Debug)]
#[br(repr = i32)]
enum StreamingMode {
  Unknown = -1,
  _1D,
  _2D,
  Manual,
}

#[derive(BinRead, Debug)]
pub struct Header {
  pub version: i32,

  flags: i32,
  sections: i32,
  collision_models: i32,
  renderable_models: i32,
  motion_packs: i32,
  streaming_textures: i32,
  static_textures: i32,
  uncached_pages: i32,
  cached_pages: i32,

  motion_packs_offset: i32,
  renderable_models_offset: i32,
  collision_models_offset: i32,
  textures_offset: i32,
  collision_grids_offset: i32,

  streaming_mode: StreamingMode,
  reserved: [u8; 16],
}

#[derive(BinRead, Debug)]
pub struct ModelInfo {
  flags: i32,
  pub position: Vector4,
  pub look_vector: Vector4,
  pub up_vector: Vector4,
  pub is_animated: i32,
  section_id: i32,
  component_id: i32,

  pub name: [u8; 260],

  zone: i32,
  pub parameter_count: i32,
}

#[derive(BinRead, Debug)]
pub struct StreamingParameter {
  name: [u8; 260],
  value: [u8; 260],
}

impl std::fmt::Display for StreamingParameter {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}={}",
      clean_string(&self.name),
      clean_string(&self.value)
    )
  }
}

#[derive(BinRead, Debug)]
pub struct StreamingTexture<TH: BinRead<Args<'static> = ()> + 'static> {
  pub model_info: ModelInfo,
  pub padding: u32,
  pub header: TH,
}

#[derive(BinRead, Debug)]
pub struct StaticTexture {
  pub model_info: ModelInfo,

  pub dds_size: u32,
  #[br(count = dds_size)]
  pub header_file: Vec<u8>,
}

#[derive(BinRead, Debug)]
pub struct Soi<TH: BinRead<Args<'static> = ()> + 'static> {
  pub header: Header,

  #[br(count = header.uncached_pages)]
  uncached_page_sizes: Vec<i32>,

  #[br(count = header.cached_pages)]
  cached_page_sizes: Vec<i32>,

  #[br(count = header.streaming_textures)]
  streaming_textures: Vec<StreamingTexture<TH>>,

  #[br(count = header.static_textures)]
  static_textures: Vec<StaticTexture>,

  #[br(count = header.motion_packs)]
  motion_packs: Vec<StreamingMotionPack>,

  // #[br(if(header.flags & 64 == 1))]
  // collision_grid_info: StreamingCollisionGridInfo,
  #[br(count = header.renderable_models)]
  renderable_models: Vec<StreamingRenderableModel>,

  #[br(count = header.collision_models)]
  collision_models: Vec<StreamingCollisionModel>,
}

impl<TH: BinRead<Args<'static> = ()>> Soi<TH> {
  pub fn read(path: &Path) -> BinResult<Self> {
    let mut file = File::open(path)?;
    Self::read_file(&mut file)
  }

  pub fn read_file(file: &mut File) -> BinResult<Self> {
    file.read_be()
  }

  pub fn get_streaming_textures(&self) -> &[StreamingTexture<TH>] {
    return &self.streaming_textures;
  }

  pub fn get_static_textures(&self) -> &[StaticTexture] {
    return &self.static_textures;
  }

  pub fn get_motion_packs(&self) -> &[StreamingMotionPack] {
    return &self.motion_packs;
  }

  pub fn get_renderable_models(&self) -> &[StreamingRenderableModel] {
    return &self.renderable_models;
  }

  pub fn get_collision_models(&self) -> &[StreamingCollisionModel] {
    return &self.collision_models;
  }

  pub fn find_static_texture(&self, section_id: u32, component_id: u32) -> Option<&StaticTexture> {
    for texture in &self.static_textures {
      let model_info = &texture.model_info;
      if model_info.section_id == section_id as i32
        && model_info.component_id == component_id as i32
      {
        return Some(&texture);
      }
    }

    None
  }

  pub fn find_streaming_texture(
    &self,
    section_id: u32,
    component_id: u32,
  ) -> Option<&StreamingTexture<TH>> {
    for texture in &self.streaming_textures {
      let model_info = &texture.model_info;
      if model_info.section_id == section_id as i32
        && model_info.component_id == component_id as i32
      {
        return Some(&texture);
      }
    }

    None
  }

  pub fn find_motion_pack(
    &self,
    section_id: u32,
    component_id: u32,
  ) -> Option<&StreamingMotionPack> {
    for motion_pack in &self.motion_packs {
      let model_info = &motion_pack.model_info;
      if model_info.section_id == section_id as i32
        && model_info.component_id == component_id as i32
      {
        return Some(&motion_pack);
      }
    }

    None
  }

  pub fn find_collision_model(
    &self,
    section_id: u32,
    component_id: u32,
  ) -> Option<&StreamingCollisionModel> {
    for collision_model in &self.collision_models {
      let model_info = &collision_model.model_info;
      if model_info.section_id == section_id as i32
        && model_info.component_id == component_id as i32
      {
        return Some(&collision_model);
      }
    }

    None
  }

  pub fn find_model(
    &self,
    section_id: u32,
    component_id: u32,
  ) -> Option<&StreamingRenderableModel> {
    for model in &self.renderable_models {
      let model_info = &model.model_info;
      if model_info.section_id == section_id as i32
        && model_info.component_id == component_id as i32
      {
        return Some(&model);
      }
    }

    None
  }
}
