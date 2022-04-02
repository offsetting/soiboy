use std::fs::{create_dir_all, File};
use std::io;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};

use binrw::BinWriterExt;
use flate2::read::ZlibDecoder;
use x_flipper_360::{Config, Format};

use crate::soi::{read_soi, Soi};
use crate::texture_header::{Dimension, TextureFormat, TextureHeader, TextureSize2D};
use crate::toc::{ComponentHeader, read_toc, Section, Toc};
use crate::toc::ComponentType::Texture;

mod soi;
mod texture_header;
mod toc;

fn extract(soi: PathBuf, toc: PathBuf, str: PathBuf) -> anyhow::Result<()> {
  let mut soi = read_soi(soi)?;
  let toc = read_toc(toc)?;

  let mut str_file = BufReader::new(File::open(str)?);

  for section in 0..2 {
    process_section(section as i32, &toc[section], &mut str_file, &mut soi, &toc)?;
  }

  Ok(())
}

fn process_section(
  i: i32,
  section: &Section,
  str: &mut BufReader<File>,
  soi: &mut Soi,
  toc: &Toc,
) -> anyhow::Result<()> {
  let zlib_header = &section.header.zlib_header;

  println!("{}", i);

  let uncached = decode_zlib_chunks(
    str,
    zlib_header.uncached_total_size,
    &zlib_header.uncached_sizes,
  )?;
  decode_components(&uncached, i, &section.uncached_components, soi, toc)?;

  let cached = decode_zlib_chunks(
    str,
    zlib_header.cached_total_size,
    &zlib_header.cached_sizes,
  )?;
  decode_components(&cached, i, &section.cached_components, soi, toc)?;

  Ok(())
}

fn decode_zlib_chunks(
  str: &mut BufReader<File>,
  _total_size: i32,
  sizes: &[i32],
) -> io::Result<Vec<u8>> {
  let mut whole_section = Vec::new(); // todo: allocate total_site

  for size in sizes {
    // reading compressed chunk
    let mut buf = vec![0; size.to_owned() as usize];
    str.read_exact(&mut buf)?;

    // decompressing chunk and appending to merged vector
    let mut decoder = ZlibDecoder::new(&buf[..]);
    let mut buf = Vec::new();

    decoder.read_to_end(&mut buf)?;
    whole_section.append(&mut buf);
  }

  Ok(whole_section)
}

fn decode_components(
  data: &[u8],
  section: i32,
  components: &[ComponentHeader],
  soi: &mut Soi,
  toc: &Toc,
) -> anyhow::Result<()> {
  for component in components {
    if component.component_type != Texture {
      continue;
    }

    let start = component.memory_entry.offset as usize;
    let end = start + component.memory_entry.size as usize;

    println!(
      "{}..{} {} - {} {}",
      start,
      end,
      data.len(),
      component.id,
      clean_string(&component.path)
    );

    let out_path = PathBuf::from(format!("./data/out/{}.dds", clean_string(&component.path).replace('\\', "/")));
    create_dir_all(out_path.parent().unwrap())?;

    let mut out = File::create(&out_path)?;

    // extracting texture header from directly from soi using section and component id
    match find_texture_header(soi, section, component.id) {
      Some(header) => {
        // println!("{}", serde_json::to_string(&header.metadata())?);
        write_dds(&data[start..end], &mut out, header);
        // out.write_be(header)?;
      }
      None => {
        // couldn't find texture using component and section id -> lets try it using instance id
        // instance ids are used is the same metadata applies for multiple textures
        match find_ids_by_instance_id(toc, component.instance_id) {
          Some((section_id, component_id)) => {
            match find_texture_header(soi, section_id, component_id) {
              Some(header) => {
                write_dds(&data[start..end], &mut out, header);
                // out.write_be(header)?;
              }
              None => panic!(
                "Unable to find texture in soi... {} {} {} {} {}",
                section, component.id, component.instance_id, start, end
              ),
            }
          }
          None => panic!(
            "Unable to find texture in soi... {} {} {} {} {}",
            section, component.id, component.instance_id, start, end
          ),
        }
      }
    };
    out.write_all(&data[start..end])?;
  }

  Ok(())
}

fn write_dds(data: &[u8],
             out: &mut File,
             header: &mut TextureHeader) {
  let metadata = &header.metadata();
  let texture_size: TextureSize2D = match metadata.dimension() {
    Dimension::TwoDOrStacked => {
      TextureSize2D::from_bytes(metadata.texture_size().to_le_bytes())
    }
    // todo: implement other dimensions
    _ => panic!("https://youtu.be/p64PeG5O2mo")
  };

  let config = Config {
    width: (texture_size.width() + 1) as u32,
    height: (texture_size.height() + 1) as u32,
    depth: None,
    pitch: metadata.pitch() as u32,
    tiled: metadata.tiled(),
    packed_mips: metadata.packed_mips(),
    format: match metadata.format() {
      TextureFormat::Dxt1 => Format::Dxt1,
      TextureFormat::Dxt4_5 => Format::Dxt5,
      _ => panic!("https://youtu.be/p64PeG5O2mo"),
    },
    mipmap_levels: Some(1.max((metadata.max_mip_level() - metadata.min_mip_level()) as u32)),
    base_address: metadata.base_address(),
    mip_address: metadata.mip_address(),
  };
  println!("anal{:?}", config.format);
  x_flipper_360::convert_to_dds(&config, &data, out);
}

fn find_ids_by_instance_id(toc: &Toc, instance_id: i32) -> Option<(i32, i32)> {
  for (index, section) in toc.iter().enumerate() {
    for comp in &section.cached_components {
      if comp.instance_id == instance_id {
        return Some((index as i32, comp.id));
      }
    }
    for comp in &section.uncached_components {
      if comp.instance_id == instance_id {
        return Some((index as i32, comp.id));
      }
    }
  }

  None
}

fn find_texture_header(
  soi: &mut Soi,
  section_id: i32,
  component_id: i32,
) -> Option<&mut TextureHeader> {
  for texture in &mut soi.streaming_textures {
    if texture.model_info.section_id == section_id
      && texture.model_info.component_id == component_id
    {
      return Some(&mut texture.header);
    }
  }

  None
}

fn clean_string(d: &[char]) -> String {
  let mut a = String::new();

  for x in d {
    if *x == '\0' {
      break;
    }

    a.push(*x)
  }

  a
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use crate::extract;

  #[test]
  fn it_works() {
    // println!("figge disch");
    extract(
      PathBuf::from("./data/VehicleInfo.x360.soi"),
      PathBuf::from("./data/VehicleInfo.x360.toc"),
      PathBuf::from("./data/VehicleInfo.x360.str"),
    )
      .unwrap();
  }
}
