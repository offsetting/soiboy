use std::fs::{create_dir_all, File};
use std::io::{BufReader, Read, Write};
use std::io;
use std::path::PathBuf;

use binrw::BinResult;
use flate2::read::ZlibDecoder;

use crate::soi::{read_soi, Soi};
use crate::toc::{ComponentHeader, read_toc, Section};
use crate::toc::ComponentType::Texture;

mod toc;
mod soi;

fn extract(soi: PathBuf, toc: PathBuf, str: PathBuf, destination: PathBuf) -> BinResult<()> {
  let mut soi = read_soi(PathBuf::from("./data/VehicleInfo.x360.soi")).unwrap();
  let sections = read_toc(toc)?;

  let mut str_file = BufReader::new(File::open(str)?);

  for section in 0..5 {
    process_section(section as i32, &sections[section], &mut str_file, &mut soi);
  }

  Ok(())
}

fn process_section(i: i32, section: &Section, str: &mut BufReader<File>, soi: &mut Soi) -> io::Result<()> {
  let zlib_header = &section.header.zlib_header;

  println!("{}", i);

  let uncached = decode_zlib_chunks(str, zlib_header.uncached_total_size, &zlib_header.uncached_sizes)?;
  decode_components(&uncached, i, &section.uncached_components, soi)?;

  let cached = decode_zlib_chunks(str, zlib_header.cached_total_size, &zlib_header.cached_sizes)?;
  decode_components(&cached, i, &section.cached_components, soi)?;

  Ok(())
}

fn decode_zlib_chunks(str: &mut BufReader<File>, total_size: i32, sizes: &[i32]) -> io::Result<Vec<u8>> {
  let mut whole_section = vec![0; total_size as usize];

  for size in sizes {
    // reading compressed chunk
    let mut buf = vec![0; size.to_owned() as usize];
    str.read_exact(&mut buf)?;

    // decompressing chunk and appending to merged vector
    let mut decoder = ZlibDecoder::new(&buf[..]);
    let mut buf = Vec::new();

    decoder.read_to_end(&mut buf);
    whole_section.append(&mut buf);
  }

  Ok(whole_section)
}

fn decode_components(data: &[u8], section: i32, components: &[ComponentHeader], soi: &mut Soi) -> io::Result<()> {
  for component in components {
    if component.component_type != Texture {
      continue;
    }

    let start = component.memory_entry.offset as usize;
    let end = start + component.memory_entry.size as usize;

    println!("{}..{} {} - {} {}", start, end, data.len(), component.id, component.path.iter().collect::<String>());

    let out_path = PathBuf::from(format!("./data/out/{}/{}", section, component.id));
    create_dir_all(out_path.parent().unwrap());

    let mut out = File::create(&out_path)?;

    match find_texture_header(soi, section, component.id) {
      Some(mut header) => out.write_all(header)?,
      None => panic!("Unable to find texture in soi...")
    }

    out.write_all(&data[start..end])?;
  }

  Ok(())
}

fn find_texture_header(soi: &mut Soi, section_id: i32, component_id: i32) -> Option<&mut [u8; 52]> {
  for mut texture in &mut soi.streaming_textures {
    if texture.model_info.section_id == section_id && texture.model_info.component_id == component_id {
      return Some(&mut texture.header);
    }
  }

  None
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use crate::extract;
  use crate::soi::read_soi;

// use crate::extract;

  #[test]
  fn it_works() {
    extract(
      PathBuf::from("./data/VehicleInfo.x360.soi"),
      PathBuf::from("./data/VehicleInfo.x360.toc"),
      PathBuf::from("data/VehicleInfo.x360.str"),
      PathBuf::from("./data/dist"),
    )
      .unwrap();
  }
}
