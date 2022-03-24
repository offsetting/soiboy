use std::fs::{create_dir_all, File};
use std::io::{BufReader, Read};
use std::io;
use std::path::PathBuf;

use binrw::BinResult;
use flate2::read::ZlibDecoder;

use crate::toc::{ComponentHeader, read_toc, Section};
use crate::toc::ComponentType::Texture;

mod toc;
mod soi;

fn extract(toc: PathBuf, str: PathBuf, destination: PathBuf) -> BinResult<()> {
  let mut str_file = BufReader::new(File::open(str)?);

  let sections = read_toc(toc)?;
  println!("Found {} sections", sections.len());

  for section in 0..5 {
    process_section(section as i32, &sections[section], &mut str_file);
  }

  Ok(())
}

fn process_section(i: i32, section: &Section, str: &mut BufReader<File>) -> io::Result<()> {
  let zlib_header = &section.header.zlib_header;

  println!("{}", i);

  let uncached = decode_zlib_chunks(str, zlib_header.uncached_total_size, &zlib_header.uncached_sizes)?;
  decode_components(&uncached, i, &section.uncached_components)?;

  let cached = decode_zlib_chunks(str, zlib_header.cached_total_size, &zlib_header.cached_sizes)?;
  decode_components(&cached, i, &section.cached_components)?;

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
    decoder.read_to_end(&mut whole_section)?;
  }

  Ok(whole_section)
}

fn decode_components(data: &[u8], section: i32, components: &[ComponentHeader]) -> io::Result<()> {
  for component in components {
    if component.component_type != Texture {
      continue;
    }

    let start = component.memory_entry.offset as usize;
    let end = start + component.memory_entry.size as usize;

    let mut comp_data = &data[start..end];

    println!("{}..{} {} - {} {}", start, end, data.len(), component.component_id, component.path.iter().collect::<String>());

    let out_path = PathBuf::from(format!("./data/out/{}/{}", section, component.component_id));
    create_dir_all(out_path.parent().unwrap());

    let mut out = File::create(&out_path)?;
    io::copy(&mut comp_data, &mut out)?;
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use crate::soi::read_soi;

  #[test]
  fn it_works() {
    // extract(
    //   PathBuf::from("./data/VehicleInfo.x360.toc"),
    //   PathBuf::from("data/VehicleInfo.x360.str"),
    //   PathBuf::from("./data/dist"),
    // )
    //   .unwrap()
    let soi = read_soi(PathBuf::from("./data/VehicleInfo.x360.soi")).unwrap();
    println!("{:#?}", soi.header);

    for x in soi.streaming_textures {
      println!("{}", x.model_info.name.iter().collect::<String>());
    }
  }
}
