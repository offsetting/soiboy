use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use binrw::io;
use flate2::read::ZlibDecoder;

use crate::toc::ComponentKind;
use crate::utils::clean_string;
use crate::{ComponentHeader, Section};

#[derive(Debug)]
pub struct SectionData {
  pub uncached: Vec<ComponentData>,
  pub cached: Vec<ComponentData>,
}

#[derive(Debug)]
pub struct ComponentData {
  pub id: u32,
  pub path: String,
  pub instance_id: u32,
  pub kind: ComponentKind,
  pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct Str {
  file: File,
}

impl Str {
  pub fn read(path: &Path) -> io::Result<Self> {
    let file = File::open(path)?;
    Ok(Self::read_file(file))
  }

  pub fn read_file(file: File) -> Self {
    Self { file }
  }

  pub fn read_section_data(&mut self, section: &Section) -> io::Result<SectionData> {
    let header = &section.header;
    let zlib = &header.zlib_header;

    self
      .file
      .seek(SeekFrom::Start(header.memory_entry.offset as u64))?;

    let uncached_data = self.decode_zlib_data(&zlib.uncached_sizes)?;
    let uncached = extract_components(&section.uncached_components, uncached_data);

    let cached_data = self.decode_zlib_data(&zlib.cached_sizes)?;
    let cached = extract_components(&section.cached_components, cached_data);

    Ok(SectionData { uncached, cached })
  }

  pub fn decode_zlib_data(&mut self, sizes: &[i32]) -> io::Result<Vec<u8>> {
    let mut whole_section = Vec::new();

    for size in sizes {
      // reading compressed chunk
      let mut buf = vec![0; *size as usize];
      self.file.read_exact(&mut buf)?;

      // decompressing chunk and appending to merged vector
      let mut decoder = ZlibDecoder::new(&buf[..]);
      decoder.read_to_end(&mut whole_section)?;
    }

    Ok(whole_section)
  }
}

fn extract_components(headers: &[ComponentHeader], data: Vec<u8>) -> Vec<ComponentData> {
  let mut components = Vec::new();

  for header in headers {
    let start = header.memory_entry.offset as usize;
    let end = (header.memory_entry.offset + header.memory_entry.size) as usize;

    let component = ComponentData {
      id: header.id as u32,
      path: clean_string(&header.path).iter().collect(),
      instance_id: header.instance_id as u32,
      kind: header.kind,
      // copy data for each component
      data: data[start..end].to_vec(),
    };

    components.push(component);
  }

  components
}
