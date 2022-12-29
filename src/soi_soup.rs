use std::path::Path;

use binrw::{BinRead, BinResult};

use crate::{
  ComponentHeader, Section, Soi, StaticTexture, StreamingCollisionModel, StreamingMotionPack,
  StreamingRenderableModel, StreamingTexture, Toc,
};

pub struct SoiSoup<TH: BinRead<Args = ()>> {
  toc: Toc,
  soi: Soi<TH>,
}

impl<TH: BinRead<Args = ()>> SoiSoup<TH> {
  pub fn cook(toc_path: &Path, soi_path: &Path) -> BinResult<Self> {
    let toc = Toc::read(toc_path)?;
    let soi = Soi::read(soi_path)?;

    Ok(Self { toc, soi })
  }

  pub fn find_sections(&self) -> &Vec<Section> {
    &self.toc.sections
  }

  pub fn find_components(&self) -> Vec<(u32, &Section, &ComponentHeader)> {
    let mut components = Vec::new();

    for (id, section) in self.toc.sections.iter().enumerate() {
      let id = id as u32;

      for component in &section.uncached_components {
        components.push((id, section, component));
      }

      for component in &section.cached_components {
        components.push((id, section, component));
      }
    }

    components
  }

  pub fn component_count(&self) -> u32 {
    let mut sum = 0;

    for section in &self.toc.sections {
      sum += section.header.total_component_count;
    }

    sum as u32
  }

  pub fn find_static_texture(
    &self,
    section_id: u32,
    component_id: u32,
    instance_id: u32,
  ) -> Option<&StaticTexture> {
    if let Some(header) = self.soi.find_static_texture(section_id, component_id) {
      return Some(header);
    }

    let (section_id, component_id) = self.toc.find_ids(instance_id)?;
    self.soi.find_static_texture(section_id, component_id)
  }

  pub fn find_streaming_texture(
    &self,
    section_id: u32,
    component_id: u32,
    instance_id: u32,
  ) -> Option<&StreamingTexture<TH>> {
    if let Some(header) = self.soi.find_streaming_texture(section_id, component_id) {
      return Some(header);
    }

    let (section_id, component_id) = self.toc.find_ids(instance_id)?;
    self.soi.find_streaming_texture(section_id, component_id)
  }

  pub fn find_motion_pack(
    &self,
    section_id: u32,
    component_id: u32,
    instance_id: u32,
  ) -> Option<&StreamingMotionPack> {
    if let Some(header) = self.soi.find_motion_pack(section_id, component_id) {
      return Some(header);
    }

    let (section_id, component_id) = self.toc.find_ids(instance_id)?;
    self.soi.find_motion_pack(section_id, component_id)
  }

  pub fn find_collision_model(
    &self,
    section_id: u32,
    component_id: u32,
    instance_id: u32,
  ) -> Option<&StreamingCollisionModel> {
    if let Some(header) = self.soi.find_collision_model(section_id, component_id) {
      return Some(header);
    }

    let (section_id, component_id) = self.toc.find_ids(instance_id)?;
    self.soi.find_collision_model(section_id, component_id)
  }

  pub fn find_model(
    &self,
    section_id: u32,
    component_id: u32,
    instance_id: u32,
  ) -> Option<&StreamingRenderableModel> {
    if let Some(header) = self.soi.find_model(section_id, component_id) {
      return Some(header);
    }

    let (section_id, component_id) = self.toc.find_ids(instance_id)?;
    self.soi.find_model(section_id, component_id)
  }
}