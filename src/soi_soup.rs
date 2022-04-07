use std::io;
use std::path::Path;

use binrw::{BinRead, BinResult};

use crate::{ComponentData, ComponentHeader, Section, SectionData, Soi, Str, Toc};

pub struct SoiSoup<TH: BinRead<Args=()>> {
  toc: Toc,
  soi: Soi<TH>,
  str: Str,
}

impl<TH: BinRead<Args=()>> SoiSoup<TH> {
  pub fn cook(toc_path: &Path, soi_path: &Path, str_path: &Path) -> BinResult<Self> {
    let toc = Toc::read(toc_path)?;
    let soi = Soi::read(soi_path)?;
    let str = Str::read(str_path)?;

    Ok(Self { toc, soi, str })
  }

  pub fn find_sections(&self) -> &Vec<Section> {
    &self.toc.sections
  }

  pub fn find_components(&self) -> Vec<(u32, &ComponentHeader)> {
    let mut components = Vec::new();

    for (id, section) in self.toc.sections.iter().enumerate() {
      let id = id as u32;

      for component in &section.uncached_components {
        components.push((id, component));
      }

      for component in &section.cached_components {
        components.push((id, component));
      }
    }

    components
  }

  pub fn find_section_data(&mut self, id: u32) -> Option<io::Result<SectionData>> {
    let section = self.toc.find_section(id)?;
    Some(self.str.read_section_data(section))
  }

  pub fn find_texture_header(&self, section_id: u32, component_id: u32, instance_id: u32) -> Option<&TH> {
    if let Some(header) = self.soi.find_texture_header(section_id, component_id) {
      return Some(header);
    }

    if let Some((section_id, component_id)) = self.toc.find_ids(instance_id) {
      return self.soi.find_texture_header(section_id, component_id);
    }

    None
  }
}
