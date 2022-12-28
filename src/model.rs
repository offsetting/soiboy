use std::str::Bytes;

use binrw::{BinRead, BinResult, BinWrite, BinrwNamedArgs};

#[derive(Default, BinRead, BinWrite, Debug)]
#[brw(big)]
pub struct Vector4 {
  x: f32,
  y: f32,
  z: f32,
  w: f32,
}

#[derive(Default, BinRead, BinWrite, Debug)]
#[brw(big)]
pub struct Vector3 {
  x: f32,
  y: f32,
  z: f32,
}

#[derive(Default, BinRead, BinWrite, Debug)]
#[brw(big)]
pub struct Vector2 {
  x: f32,
  y: f32,
}

#[derive(BinRead, BinWrite, Debug)]
#[brw(big)]
pub struct XNGMeshName {
  #[br(count = 64)]
  name: Vec<u8>,
}

#[derive(BinRead, BinWrite, Debug)]
#[brw(big)]
pub struct XNGBone {
  name: [u8; 128],
  matrix: [f32; 16],
  bounding_box_center: [f32; 3],
  bounding_box_half: [f32; 3],
  bounding_box_radius: f32,
  parent_index: u32,
}

#[derive(Default, BinRead, BinWrite, Debug)]
#[brw(big)]
pub struct XNGDeltaBlock {
  num_channels: u32,

  #[br(count = 64)]
  controller_name: Vec<u8>,

  num_vertices: u32,
  xyz_bits: u32,
  force_unique: u8,
  unk: u32,
  unk2: u32,

  delta_count: u32,

  #[br(count = delta_count)]
  delta_positions: Vec<Vector4>,

  #[br(count = delta_count)]
  delta_normals: Vec<Vector4>,

  #[br(count = delta_count)]
  delta_indices: Vec<i32>,

  #[br(count = num_vertices)]
  positions: Vec<Vector4>,

  #[br(count = num_vertices)]
  normals: Vec<Vector4>,
}

#[derive(BinRead, BinWrite, Debug)]
#[brw(big)]
pub struct StreamingXNGMesh {
  pub surface_index: u32,
  pub vertex_type: u32,

  #[br(if((vertex_type & 0x2000) == 0x2000))]
  pub compression_stuff: Option<[f32; 8]>,

  pub num_texture_coordinate_sets: u8,
  pub compressed: u8,
  pub streaming: u8,
  pub unk: u8,
  pub unk2: u8,
  pub unk3: u8,

  #[br(count = num_texture_coordinate_sets)]
  pub texture_coordinate_sets: Vec<f32>,

  pub num_vertices: u16,
  pub num_face_indices: u16,

  #[br(if((vertex_type & 0x100) == 0x100))]
  pub delta_block: Option<XNGDeltaBlock>,
}

#[derive(BinRead, BinWrite, Debug)]
#[brw(big)]
pub struct XNGLod {
  pub auto_lod_value: f32,
  pub num_meshes: u32,

  #[br(count = num_meshes)]
  pub meshes: Vec<StreamingXNGMesh>,
}

#[derive(BinRead, Debug)]
#[br(big)]
#[br(magic = b"xgs\0")]
pub struct XNGHeader {
  version: i32,
  num_bones: u32,

  #[br(count = num_bones)]
  bones: Vec<XNGBone>,

  num_mesh_names: i32,

  #[br(count = num_mesh_names)]
  pub mesh_names: Vec<XNGMeshName>,

  pub num_lod: u8,
  skin_animates_flag: u8,
  has_weight: u8,
  unused: u8,

  #[br(count = num_lod)]
  pub lods: Vec<XNGLod>,
}

#[derive(BinRead, Debug)]
pub struct StreamingRenderableModel {
  pub model_info: crate::ModelInfo,

  #[br(count = model_info.parameter_count)]
  pub parameters: Vec<crate::StreamingParameter>,

  pub streaming_model_header: XNGHeader,
}

#[derive(BinrwNamedArgs, Clone, Debug)]
pub struct XNGHeaderArgs {
  pub streaming_data: Vec<u8>,
}

// This BinWrite implementation actually restructures the streaming component data plus the header data in the SOI to form a proper XNG file.
// As such, the streaming data must be passed to write_options.
impl BinWrite for XNGHeader {
  type Args = XNGHeaderArgs;

  fn write_options<W: std::io::Write + std::io::Seek>(
    &self,
    writer: &mut W,
    options: &binrw::WriteOptions,
    args: Self::Args,
  ) -> binrw::BinResult<()> {
    let magic = b"xng\0".to_vec();
    Vec::write_options(&magic, writer, options, ())?;
    i32::write_options(&self.version, writer, options, ())?;
    u32::write_options(&self.num_bones, writer, options, ())?;
    Vec::write_options(&self.bones, writer, options, ())?;
    i32::write_options(&self.num_mesh_names, writer, options, ())?;
    Vec::write_options(&self.mesh_names, writer, options, ())?;
    u8::write_options(&self.num_lod, writer, options, ())?;
    u8::write_options(&self.skin_animates_flag, writer, options, ())?;
    u8::write_options(&self.has_weight, writer, options, ())?;
    u8::write_options(&self.unused, writer, options, ())?;

    let mut offset_in_data: usize = 0;

    for lod in &self.lods {
      f32::write_options(&lod.auto_lod_value, writer, options, ())?;
      u32::write_options(&lod.num_meshes, writer, options, ())?;
      for mesh in &lod.meshes {
        u32::write_options(&mesh.surface_index, writer, options, ())?;
        u32::write_options(&mesh.vertex_type, writer, options, ())?;
        if let Some(compression_stuff) = mesh.compression_stuff {
          compression_stuff.write_options(writer, options, ())?;
        }
        u8::write_options(&mesh.num_texture_coordinate_sets, writer, options, ())?;
        u8::write_options(&mesh.compressed, writer, options, ())?;
        u8::write_options(&mesh.streaming, writer, options, ())?;
        u8::write_options(&mesh.unk, writer, options, ())?;
        u8::write_options(&mesh.unk2, writer, options, ())?;
        u8::write_options(&mesh.unk3, writer, options, ())?;
        Vec::write_options(&mesh.texture_coordinate_sets, writer, options, ())?;

        u16::write_options(&mesh.num_vertices, writer, options, ())?;
        u16::write_options(&mesh.num_face_indices, writer, options, ())?;

        assert!(mesh.streaming == 1);

        let ty = mesh.vertex_type;
        let mut offset: usize = mesh.num_face_indices as usize * 2;
        if mesh.num_face_indices % 2 == 1 {
          offset += 2;
        }
        if (ty & 0x01) == 0x01 {
          offset += (mesh.num_vertices as usize * 12);
        }
        if (ty & 0x02) == 0x02 {
          offset += (mesh.num_vertices as usize * 12);
        }
        if (ty & 0x08) == 0x08 {
          offset += (mesh.num_vertices as usize * 4);
        }
        if (ty & 0x04) == 0x04 {
          offset += (mesh.num_vertices as usize * 8);
        }
        if (ty & 0x40) == 0x40 {
          offset += (mesh.num_vertices as usize * 4);
        }
        if (ty & 0x1000) == 0x1000 {
          offset += (mesh.num_vertices as usize * 32);
        }
        if (ty & 0x10) == 0x10 {
          offset += (mesh.num_vertices as usize * 8);
        }
        if (ty & 0x4000) == 0x4000 {
          offset += (mesh.num_vertices as usize * 8);
        }
        if (ty & 0x8000) == 0x8000 {
          offset += (mesh.num_vertices as usize * 8);
        }
        if (ty & 0x20) == 0x20 {
          offset += (mesh.num_vertices as usize * 12);
        }

        let data = (&args.streaming_data[offset_in_data..offset_in_data + offset]).to_vec();
        Vec::write_options(&data, writer, options, ())?;

        offset_in_data += offset;

        if let Some(delta_block) = &mesh.delta_block {
          delta_block.write_options(writer, options, ())?;
        }
      }
    }
    Ok(())
  }
}
