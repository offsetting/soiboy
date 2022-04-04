use std::path::Path;

use crate::ComponentKind::Texture;
use crate::{ComponentData, Soi, Str, Toc};

#[test]
fn extract() {
  let toc_path = Path::new("./data/VehicleInfo.x360.toc");
  let soi_path = Path::new("./data/VehicleInfo.x360.soi");
  let str_path = Path::new("data/VehicleInfo.x360.str");

  let toc = Toc::read(toc_path).unwrap();
  let mut soi = Soi::read(soi_path).unwrap();
  let mut str = Str::read(str_path).unwrap();

  for (id, section) in toc.sections[0..5].iter().enumerate() {
    let section_data = str.read_section_data(section).unwrap();

    for component in section_data.uncached {
      process_component(&toc, &mut soi, id as u32, component);
    }

    for component in section_data.cached {
      process_component(&toc, &mut soi, id as u32, component);
    }
  }
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
