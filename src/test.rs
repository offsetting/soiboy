use std::io::Write;
use std::path::{Path, PathBuf};

use binrw::{BinWrite, WriteOptions};
use x_flipper_360::*;

use crate::utils::*;
use crate::ComponentKind::{self, *};
use crate::{CollisionModelArgs, ComponentData, SoiSoup, Str, XNGHeaderArgs};

#[test]
fn extract() {
  let toc_path = Path::new("./data/VehicleInfo.x360.toc");
  let soi_path = Path::new("./data/VehicleInfo.x360.soi");
  let str_path = Path::new("data/VehicleInfo.x360.str");

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

  for static_texture in soup.static_textures().iter() {
    let path = PathBuf::from(format!(
      "D:\\GigaLeak\\Rs_A_Mn\\Out\\{}.dds",
      clean_path(&static_texture.model_info.name)
    ));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut out = std::fs::File::create(path).unwrap();
    out.write_all(&static_texture.header_file).unwrap();
  }
}

#[test]
fn dump_scn() {
  let toc_path =
    Path::new("D:\\Xbox 360\\RoRX360_Extracted\\RELEASE (NEW)\\C\\Scenes\\CR_04\\CR_04.x360.toc");
  let soi_path =
    Path::new("D:\\Xbox 360\\RoRX360_Extracted\\RELEASE (NEW)\\C\\Scenes\\CR_04\\CR_04.x360.soi");
  let str_path =
    Path::new("D:\\Xbox 360\\RoRX360_Extracted\\RELEASE (NEW)\\C\\Scenes\\CR_04\\CR_04.x360.str");
  let soup = SoiSoup::cook(toc_path, soi_path).unwrap();
  let mut str = Str::read(str_path).unwrap();
  let mut num_anim_models = 1;
  let mut num_static_models = 1;
  let mut num_objects = 1;
  for (id, section) in soup.find_sections().iter().enumerate() {
    let section_data = str.read_section_data(section).unwrap();

    for component in section_data.uncached {
      print_component(
        &soup,
        id as u32,
        component,
        &mut num_anim_models,
        &mut num_static_models,
        &mut num_objects,
      );
    }

    for component in section_data.cached {
      print_component(
        &soup,
        id as u32,
        component,
        &mut num_anim_models,
        &mut num_static_models,
        &mut num_objects,
      );
    }
  }
}

fn print_component(
  soup: &SoiSoup<TextureHeader>,
  section_id: u32,
  component: ComponentData,
  num_anim_models: &mut i32,
  num_static_models: &mut i32,
  num_objects: &mut i32,
) {
  match component.kind {
    RenderableModel => {
      let header = soup
        .find_model(section_id, component.id, component.instance_id)
        .unwrap();
      if header.model_info.is_animated == 1 {
        println!("[AnimatedModel{}]\n{}", num_anim_models, header);
        *num_anim_models = *num_anim_models + 1;
      } else {
        println!("[Model{}]\n{}", num_static_models, header);
        *num_static_models = *num_static_models + 1;
      }
    }
    Texture => {
      // println!("found Texture component kind; skipping...");
    }
    CollisionModel => {
      let header = soup
        .find_collision_model(section_id, component.id, component.instance_id)
        .unwrap();
      println!("[Object{}]\n{}", num_objects, header);
      *num_objects = *num_objects + 1;
    }
    UserData => {
      // println!("found UserData component kind; skipping...");
    }
    MotionPack => {
      // println!("found MotionPack component kind; skipping...");
    }
    CollisionGrid => {
      // println!("found CollisionGrid component kind; skipping...");
    }
  }
}

fn process_component(soup: &SoiSoup<TextureHeader>, section_id: u32, component: ComponentData) {
  if component.kind == ComponentKind::MotionPack {
    let header = soup
      .find_motion_pack(section_id, component.id, component.instance_id)
      .unwrap();

    let path = PathBuf::from(format!(
      "D:\\GigaLeak\\Rs_A_Mn\\Out\\{}.got",
      component.path
    ));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut out = std::fs::File::create(path).unwrap();
    header.header.write(&mut out);
    out.write_all(&component.data);
  }

  if component.kind == ComponentKind::RenderableModel {
    let header = soup
      .find_model(section_id, component.id, component.instance_id)
      .unwrap();

    let path = PathBuf::from(format!(
      "D:\\GigaLeak\\Rs_A_Mn\\Out\\{}.xng",
      component.path
    ));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut out = std::fs::File::create(path).unwrap();
    let options = WriteOptions::new(binrw::Endian::Big);
    header
      .streaming_model_header
      .write_options(
        &mut out,
        &options,
        XNGHeaderArgs {
          streaming_data: component.data.clone(),
        },
      )
      .unwrap();
  }
  if component.kind == ComponentKind::CollisionModel {
    let header = soup
      .find_collision_model(section_id, component.id, component.instance_id)
      .unwrap();

    let path = PathBuf::from(format!(
      "D:\\GigaLeak\\Rs_A_Mn\\Out\\{}.gol",
      component.path
    ));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut out = std::fs::File::create(path).unwrap();
    let options = WriteOptions::new(binrw::Endian::Big);
    header
      .collision_model
      .write_options(
        &mut out,
        &options,
        CollisionModelArgs {
          ror: false,
          streaming_data: component.data.clone(),
        },
      )
      .unwrap();
  }
  if component.kind == ComponentKind::Texture {
    match soup.find_streaming_texture(section_id, component.id, component.instance_id) {
      Some(header) => {
        let metadata = header.header.metadata();

        match metadata.format() {
          TextureFormat::Dxt1 => {
            let texture_size: TextureSize2D =
              TextureSize2D::from_bytes(metadata.texture_size().to_le_bytes());

            let config = Config {
              width: texture_size.width() as u32 + 1,
              height: texture_size.height() as u32 + 1,
              depth: None,
              pitch: metadata.pitch() as u32,
              tiled: metadata.tiled(),
              packed_mips: metadata.packed_mips(),
              format: Format::Dxt1,
              mipmap_levels: Some(1.max(metadata.max_mip_level() - metadata.min_mip_level()) as u32),
              base_address: metadata.base_address(),
              mip_address: metadata.mip_address(),
            };

            let path = PathBuf::from(format!(
              "D:\\GigaLeak\\Rs_A_Mn\\Out\\{}.dds",
              component.path
            ));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            let mut out = std::fs::File::create(path).unwrap();
            x_flipper_360::convert_to_dds(&config, &component.data, &mut out).unwrap();
          }
          TextureFormat::Dxt4_5 => {
            let texture_size: TextureSize2D =
              TextureSize2D::from_bytes(metadata.texture_size().to_le_bytes());

            let config = Config {
              width: texture_size.width() as u32 + 1,
              height: texture_size.height() as u32 + 1,
              depth: None,
              pitch: metadata.pitch() as u32,
              tiled: metadata.tiled(),
              packed_mips: metadata.packed_mips(),
              format: Format::Dxt5,
              mipmap_levels: Some(1.max(metadata.max_mip_level() - metadata.min_mip_level()) as u32),
              base_address: metadata.base_address(),
              mip_address: metadata.mip_address(),
            };

            let path = PathBuf::from(format!(
              "D:\\GigaLeak\\Rs_A_Mn\\Out\\{}.dds",
              component.path
            ));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            let mut out = std::fs::File::create(path).unwrap();
            x_flipper_360::convert_to_dds(&config, &component.data, &mut out).unwrap();
          }
          TextureFormat::Dxt2_3 => {
            let texture_size: TextureSize2D =
              TextureSize2D::from_bytes(metadata.texture_size().to_le_bytes());

            let config = Config {
              width: texture_size.width() as u32 + 1,
              height: texture_size.height() as u32 + 1,
              depth: None,
              pitch: metadata.pitch() as u32,
              tiled: metadata.tiled(),
              packed_mips: metadata.packed_mips(),
              format: Format::Dxt3,
              mipmap_levels: Some(1.max(metadata.max_mip_level() - metadata.min_mip_level()) as u32),
              base_address: metadata.base_address(),
              mip_address: metadata.mip_address(),
            };

            let path = PathBuf::from(format!(
              "D:\\GigaLeak\\Rs_A_Mn\\Out\\{}.dds",
              component.path
            ));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            let mut out = std::fs::File::create(path).unwrap();
            x_flipper_360::convert_to_dds(&config, &component.data, &mut out).unwrap();
          }
          _ => {
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

            let path = PathBuf::from(format!(
              "D:\\GigaLeak\\Rs_A_Mn\\Out\\{}.dds",
              component.path
            ));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            let mut out = std::fs::File::create(path).unwrap();
            x_flipper_360::convert_to_dds(&config, &component.data, &mut out).unwrap();
          }
        }
      }
      None => match soup.find_static_texture(section_id, component.id, component.instance_id) {
        Some(static_texture) => {
          let path = PathBuf::from(format!(
            "D:\\GigaLeak\\Rs_A_Mn\\Out\\{}.dds",
            component.path
          ));
          std::fs::create_dir_all(path.parent().unwrap()).unwrap();
          let mut out = std::fs::File::create(path).unwrap();
          out.write_all(&static_texture.header_file).unwrap();
        }
        None => panic!("Failed to find texture header."),
      },
    }
  }
}
