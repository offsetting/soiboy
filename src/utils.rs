use binrw::{BinRead, BinWrite};

#[derive(Default, BinRead, BinWrite, Debug)]
#[brw(big)]
pub struct Vector4 {
  pub x: f32,
  pub y: f32,
  pub z: f32,
  pub w: f32,
}

const NULL_BYTE: u8 = b'\0';
const BACKSLASH: u8 = b'\\';
const SLASH: char = '/';

impl std::fmt::Display for Vector4 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{},{},{},{}", self.x, self.y, self.z, self.w)
  }
}

#[derive(Default, BinRead, BinWrite, Debug, Clone, Copy)]
#[brw(big)]
pub struct Vector3 {
  pub x: f32,
  pub y: f32,
  pub z: f32,
}

impl std::fmt::Display for Vector3 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{},{},{}", self.x, self.y, self.z)
  }
}

#[derive(Default, BinRead, BinWrite, Debug)]
#[brw(big)]
pub struct Vector2 {
  pub x: f32,
  pub y: f32,
}

impl std::fmt::Display for Vector2 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{},{}", self.x, self.y)
  }
}

#[derive(BinRead, BinWrite, Debug)]
#[brw(big)]
pub struct Vector3i16 {
  pub x: i16,
  pub y: i16,
  pub z: i16,
}

#[derive(BinRead, BinWrite, Debug)]
#[brw(big)]
pub struct Vector4i16 {
  pub x: i16,
  pub y: i16,
  pub z: i16,
  pub w: i16,
}

pub(crate) fn clean_path(input: &[u8]) -> String {
  let mut output = String::new();

  for c in input {
    if c == &NULL_BYTE {
      return output;
    }

    if c == &BACKSLASH {
      output.push(SLASH);
    } else {
      output.push(*c as char)
    }
  }

  output
}

pub fn clean_string(input: &[u8]) -> String {
  let mut output = Vec::new();

  for c in input {
    if c == &NULL_BYTE {
      return std::str::from_utf8(&output).unwrap().to_owned();
    }
    output.push(*c)
  }

  std::str::from_utf8(&output).unwrap().to_owned()
}
