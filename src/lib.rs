use std::path::Path;
use crate::ComponentKind::Texture;

pub use crate::soi::*;
pub use crate::str::*;
pub use crate::toc::*;

mod soi;
mod str;
mod texture_header;
mod toc;
mod utils;

pub struct Config<'a, 'b, 'c> {
  pub toc: &'b Path,
  pub soi: &'a Path,
  pub str: &'c Path,
}

fn extract(config: &Config) -> anyhow::Result<()> {
  let toc = Toc::read(config.toc)?;
  let mut soi = Soi::read(config.soi)?;
  let mut str = Str::read(config.str)?;

  for (id, section) in toc.sections[0..5].iter().enumerate() {
    let section_data = str.read_section_data(section)?;

    for component in section_data.uncached {
      process_component(&toc, &mut soi, id as u32, component);
    }

    for component in section_data.cached {
      process_component(&toc, &mut soi, id as u32, component);
    }
  }

  Ok(())
}

fn process_component(toc: &Toc, soi: &mut Soi, section_id: u32, component: ComponentData) {
  if component.kind != Texture {
    return;
  }

  let header = match soi.find_texture_header(section_id, component.id) {
    Some(header) => header,
    None => match toc.find_ids(component.instance_id) {
      Some((section_id, component_id)) => match soi.find_texture_header(section_id, component_id) {
        Some(header) => header,
        None => panic!("can not find texture header by section and component id"),
      },
      None => panic!("can not find texture header by section and component id nor instance id"),
    },
  };

  let metadata = header.metadata();
  println!("{} {:?}", component.path, metadata.format());
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::path::{Path, PathBuf};

  use crate::{extract, Config};

  #[test]
  fn it_works() {
    let config = Config {
      soi: Path::new("./data/VehicleInfo.x360.soi"),
      toc: Path::new("./data/VehicleInfo.x360.toc"),
      str: Path::new("data/VehicleInfo.x360.str"),
    };

    extract(&config).unwrap();
  }
}
