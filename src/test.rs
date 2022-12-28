use std::io::Write;
use std::path::{Path, PathBuf};

use binrw::{BinWrite, WriteOptions};
use x_flipper_360::*;

use crate::ComponentKind::{self, Texture};
use crate::{ComponentData, SoiSoup, Str, XNGHeaderArgs};

#[test]
fn extract() {
  let toc_path =
    Path::new("D:\\Xbox 360\\RoRX360_Extracted\\RELEASE (NEW)\\C\\Scenes\\HUB\\HUB.x360.toc");
  let soi_path =
    Path::new("D:\\Xbox 360\\RoRX360_Extracted\\RELEASE (NEW)\\C\\Scenes\\HUB\\HUB.x360.soi");
  let str_path =
    Path::new("D:\\Xbox 360\\RoRX360_Extracted\\RELEASE (NEW)\\C\\Scenes\\HUB\\HUB.x360.str");

  let soup = SoiSoup::cook(toc_path, soi_path).unwrap();
  let mut str = Str::read(str_path).unwrap();

  for (id, section) in soup.find_sections().iter().enumerate() {
    let section_data = str.read_section_data(section).unwrap();

    for component in section_data.uncached {
      process_component(&soup, id as u32, component);
    }

    for component in section_data.cached {
      process_component(&soup, id as u32, component);
    }
  }
}

fn process_component(soup: &SoiSoup<TextureHeader>, section_id: u32, component: ComponentData) {
  if component.kind == ComponentKind::MotionPack {
    let header = soup
      .find_motion_pack(section_id, component.id, component.instance_id)
      .unwrap();

    let path = PathBuf::from(format!("D:\\GigaLeak\\asdjkl\\{}.got", component.path));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut out = std::fs::File::create(path).unwrap();
    header.write_to(&mut out);
    out.write_all(&component.data);
  }

  if component.kind == ComponentKind::RenderableModel {
    let header = soup
      .find_model(section_id, component.id, component.instance_id)
      .unwrap();

    let path = PathBuf::from(format!("D:\\GigaLeak\\asdjkl\\{}.xng", component.path));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut out = std::fs::File::create(path).unwrap();
    let op = WriteOptions::new(binrw::Endian::Big);
    println!(
      "{}, {}, {}, {}, {}",
      component.path,
      component.data.len(),
      header.mesh_names.len(),
      header.num_lod,
      header.lods[0].num_meshes
    );
    header.write_options(
      &mut out,
      &op,
      XNGHeaderArgs {
        streaming_data: component.data,
      },
    );

    // out.write_all(&component.data);
  }

  if component.kind == ComponentKind::Texture {
    match soup.find_texture_header(section_id, component.id, component.instance_id) {
      Some(header) => {
        let metadata = header.metadata();
        println!("{} {:?}", component.path, metadata.format());

        match metadata.format() {
          TextureFormat::Dxt1 => {}
          TextureFormat::Dxt4_5 => {}
          _ => {
            println!("{} {:?}", component.path, metadata.format());

            let texture_size: TextureSize2D =
              TextureSize2D::from_bytes(metadata.texture_size().to_le_bytes());

            let config = Config {
              width: texture_size.width() as u32 + 1,
              height: texture_size.height() as u32 + 1,
              depth: None,
              pitch: metadata.pitch() as u32,
              tiled: metadata.tiled(),
              packed_mips: metadata.packed_mips(),
              format: Format::RGBA8,
              mipmap_levels: Some(1.max(metadata.max_mip_level() - metadata.min_mip_level()) as u32),
              base_address: metadata.base_address(),
              mip_address: metadata.mip_address(),
            };

            let path = PathBuf::from(format!("D:\\GigaLeak\\asdjkl\\{}.dds", component.path));
            //std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            //let mut out = std::fs::File::create(path).unwrap();

            //x_flipper_360::convert_to_dds(&config, &component.data, &mut out).unwrap();
          }
        }
      }
      None => {
        match soup.find_static_texture_header(section_id, component.id, component.instance_id) {
          Some(static_header) => {
            let path = PathBuf::from(format!("D:\\GigaLeak\\asdjkl\\{}.dds", component.path));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            let mut out = std::fs::File::create(path).unwrap();
            out.write_all(static_header).unwrap();
          }
          None => panic!("Bruh Moment!"),
        }
      }
    }
  }
}
