use binrw::{BinRead, BinResult, BinWrite, Endian};
use std::io::{Seek, Write};

use crate::utils::*;

// Members that are commented out are part of the streaming data section, and need to be merged into the contents of the header data after extraction.

#[derive(BinRead, BinWrite, PartialEq, Debug)]
#[brw(repr = i32)]
enum CollisionType {
  Soultree = 0,
  SoultreeHeirarchy,
  Rays,
  DynamicRays,
  RadiusedLine,
  Sphere,
  Box,
  Ecosystem,
  FinitePlane,

  StreamingSoultree,
  StreamingHeirarchy,
  StreamingFinitePlane,
}

impl std::fmt::Display for CollisionType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      CollisionType::Soultree => write!(f, "Soultree"),
      CollisionType::SoultreeHeirarchy => write!(f, "SoultreeHeirarchy"),
      CollisionType::Rays => write!(f, "Rays"),
      CollisionType::DynamicRays => write!(f, "DynamicRays"),
      CollisionType::RadiusedLine => write!(f, "RadiusedLine"),
      CollisionType::Sphere => write!(f, "Sphere"),
      CollisionType::Box => write!(f, "Box"),
      CollisionType::Ecosystem => write!(f, "Ecosystem"),
      CollisionType::FinitePlane => write!(f, "FinitePlane"),
      CollisionType::StreamingSoultree => write!(f, "StreamingSoultree"),
      CollisionType::StreamingHeirarchy => write!(f, "StreamingHeirarchy"),
      CollisionType::StreamingFinitePlane => write!(f, "StreamingFinitePlane"),
    }
  }
}

#[derive(Default, BinRead, BinWrite, Debug)]
#[brw(big)]
struct TreeFace {
  volume: f32,
  vectors: [Vector3; 2],
  type_indices: [i16; 2],
}

#[derive(Default, BinRead, BinWrite, Debug)]
#[brw(big)]
struct RaceORamaStreamingDataTreeFace {
  vectors: [Vector4; 2],
  type_indices: [i16; 2],
  volume: f32,
  #[brw(pad_after = 20)]
  radius: f32,
}

impl RaceORamaStreamingDataTreeFace {
  pub fn to_tree_face(&self) -> TreeFace {
    TreeFace {
      volume: self.volume,
      vectors: [
        Vector3 {
          x: self.vectors[0].x,
          y: self.vectors[0].y,
          z: self.vectors[0].z,
        },
        Vector3 {
          x: self.vectors[1].x,
          y: self.vectors[1].y,
          z: self.vectors[1].z,
        },
      ],
      type_indices: self.type_indices,
    }
  }
}

#[derive(Default, BinRead, BinWrite, Debug)]
#[brw(big)]
struct StreamingDataTreeFace {
  volume: f32,
  radius: f32,
  type_indices: [i16; 2],
  vectors: [Vector3; 2],
}

impl StreamingDataTreeFace {
  pub fn to_tree_face(&self) -> TreeFace {
    TreeFace {
      volume: self.volume,
      vectors: self.vectors,
      type_indices: self.type_indices,
    }
  }
}

#[derive(BinRead, BinWrite, Debug)]
#[brw(big)]
struct TreeFaceLeaf {
  dvalue: f32,
  vector: Vector3,
  vertices: [i16; 3],
}

#[derive(BinRead, BinWrite, Debug)]
#[brw(big)]
struct RaceORamaStreamingDataTreeFaceLeaf {
  vector: Vector4,
  dvalue: f32,
  #[brw(pad_after = 6)]
  vertices: [i16; 3],
}

impl RaceORamaStreamingDataTreeFaceLeaf {
  pub fn to_tree_face_leaf(&self) -> TreeFaceLeaf {
    TreeFaceLeaf {
      dvalue: self.dvalue,
      vector: Vector3 {
        x: self.vector.x,
        y: self.vector.y,
        z: self.vector.z,
      },
      vertices: self.vertices,
    }
  }
}

#[derive(BinRead, BinWrite, Debug)]
#[brw(big)]
struct StreamingDataTreeFaceLeaf {
  vertices: [i16; 3],
  unknown1: f32,
  unknown2: u16,
  vector: Vector3i16,
  unknown1_bits: u16,
}
// assert_eq!(std::mem::size_of<StreamingDataTreeFaceLeaf>, 20);

impl StreamingDataTreeFaceLeaf {
  pub fn to_tree_face_leaf(&self, global_vertices: &[Vector3]) -> TreeFaceLeaf {
    let vertex = global_vertices.get(self.vertices[0] as usize).unwrap();
    let normal = Vector3 {
      x: self.vector.x as f32 / 16384.0_f32,
      y: self.vector.y as f32 / 16384.0_f32,
      z: self.vector.z as f32 / 16384.0_f32,
    };
    let dvalue = -1.0_f32 * (vertex.x * normal.x + vertex.y * normal.y + vertex.z * normal.z);
    TreeFaceLeaf {
      dvalue,
      vector: normal,
      vertices: [
        self.vertices[0] as i16,
        self.vertices[1] as i16,
        self.vertices[2] as i16,
      ],
    }
  }
}

#[derive(Default, BinRead, BinWrite, Debug)]
#[brw(big)]
struct SoultreeCollisionObject {
  temp_cmt: i32,

  obb_data: [f32; 12],
  reverse_collision_mode: i32,
  vertex_count: u32,
  tree_face_count: u32,
  tree_face_leaf_count: u32,
  quantized: i32,

  // #[br(if(quantized == 1), count = vertex_count)]
  // quantized_tree_vertices: Vec<Vector4i16>,

  // #[br(if(quantized == 0), count = vertex_count)]
  // tree_vertices: Vec<Vector4>,
  load_normals: i32,

  // #[br(if(load_normals == 1 && quantized == 1), count = vertex_count)]
  // quantized_tree_normals: Vec<Vector4i16>,

  // #[br(if(load_normals == 1 && quantized == 0), count = vertex_count)]
  // tree_normals: Vec<Vector4>,

  // #[br(count = tree_face_count)]
  // tree_faces: Vec<TreeFace>,
  top_tree_face: TreeFace,
  // #[br(count = tree_face_leaf_count)]
  // tree_face_leaves: Vec<TreeFaceLeaf>,
}

#[derive(BinRead, BinWrite, Debug)]
#[brw(big)]
struct FinitePlaneStruct {
  local_vertex_bl: Vector3,
  local_vertex_br: Vector3,
  local_vertex_tl: Vector3,
  local_vertex_tr: Vector3,
  plane_normal: Vector3,
}

#[derive(BinRead, BinWrite, Debug)]
#[brw(big)]
struct StreamingHeirarchyEntry {
  object_id: i32,
  object: SoultreeCollisionObject,
}

#[derive(BinRead, Debug)]
#[brw(big)]
#[br(magic = b"\x00\x00\x04\xD2")]
pub struct CollisionModel {
  col_type: [u8; 4],
  version: i32,
  collision_type: CollisionType,

  #[br(if (collision_type == CollisionType::StreamingSoultree))]
  object: SoultreeCollisionObject,

  #[br(if (collision_type == CollisionType::StreamingHeirarchy))]
  object_count: i32,
  #[br(if (collision_type == CollisionType::StreamingHeirarchy))]
  reverse_collision_mode: i32,
  #[br(if (collision_type == CollisionType::StreamingHeirarchy), count = object_count)]
  objects: Vec<StreamingHeirarchyEntry>,

  #[br(if (collision_type == CollisionType::StreamingFinitePlane))]
  plane_count: i32,
  #[br(if (collision_type == CollisionType::StreamingFinitePlane))]
  half: Vector3,
  // #[br(if(collision_type == CollisionType::FinitePlane), count = plane_count)]
  // planes: Vec<FinitePlaneStruct>,
}

#[derive(BinRead, Debug)]
pub struct StreamingCollisionModel {
  pub model_info: crate::ModelInfo,

  #[br(count = model_info.parameter_count)]
  pub parameters: Vec<crate::StreamingParameter>,

  pub collision_model: CollisionModel,
}

impl std::fmt::Display for StreamingCollisionModel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "COL={}.col\nPosition={}\nLookVector={}\nUpVector={}\n",
      clean_string(&self.model_info.name),
      self.model_info.position,
      self.model_info.look_vector,
      self.model_info.up_vector
    );
    Ok(for param in self.parameters.iter() {
      write!(f, "{}\n", param);
    })
  }
}

// Derive BinrwNamedArgs
#[derive(Clone, Debug)]
pub struct CollisionModelArgs {
  pub ror: bool,
  pub streaming_data: Vec<u8>,
}

// This BinWrite implementation actually restructures the streaming component data plus the header data in the SOI to form a proper GOL file.
// As such, the streaming data must be passed to write_options.
impl BinWrite for CollisionModel {
  type Args<'a> = &'a CollisionModelArgs;

  fn write_options<W: Write + Seek>(
    &self,
    writer: &mut W,
    endian: Endian,
    args: Self::Args<'_>,
  ) -> BinResult<()> {
    let magic = b"\x00\x00\x04\xD2".to_vec();
    Vec::write_options(&magic, writer, endian, ())?;
    self.col_type.write_options(writer, endian, ())?;
    i32::write_options(&self.version, writer, endian, ())?;

    let mut offset_in_data: usize = 0;

    let mut cursor = std::io::Cursor::new(&args.streaming_data);

    match self.collision_type {
      CollisionType::Soultree => panic!("Unsupported collision type!"),
      CollisionType::SoultreeHeirarchy => panic!("Unsupported collision type!"),
      CollisionType::Rays => panic!("Unsupported collision type!"),
      CollisionType::DynamicRays => panic!("Unsupported collision type!"),
      CollisionType::RadiusedLine => panic!("Unsupported collision type!"),
      CollisionType::Sphere => panic!("Unsupported collision type!"),
      CollisionType::Box => panic!("Unsupported collision type!"),
      CollisionType::Ecosystem => panic!("Unsupported collision type!"),
      CollisionType::FinitePlane => panic!("Unsupported collision type!"),
      CollisionType::StreamingSoultree => {
        CollisionType::write_options(&CollisionType::Soultree, writer, endian, ())?;

        i32::write_options(&self.object.temp_cmt, writer, endian, ())?;
        self.object.obb_data.write_options(writer, endian, ())?;
        i32::write_options(&self.object.reverse_collision_mode, writer, endian, ())?;
        u32::write_options(&self.object.vertex_count, writer, endian, ())?;
        u32::write_options(&self.object.tree_face_count, writer, endian, ())?;
        u32::write_options(&self.object.tree_face_leaf_count, writer, endian, ())?;
        i32::write_options(&self.object.quantized, writer, endian, ())?;

        // Conversion from MN's streaming format requires the vertices be cached and used later for DValue calculation.
        let mut global_vertices = Vec::new();

        if self.object.quantized == 0 {
          if args.ror {
            // Read Vector4s from the data and write out Vector3s, truncating the W component.
            cursor.set_position(offset_in_data as u64);
            let vertices = Vec::<Vector4>::read_options(
              &mut cursor,
              binrw::Endian::Big,
              binrw::VecArgs::builder()
                .count(self.object.vertex_count as usize)
                .finalize(),
            )
            .unwrap();

            let mut truncated_vertices = Vec::new();
            for vec in vertices.iter() {
              truncated_vertices.push(Vector3 {
                x: vec.x,
                y: vec.y,
                z: vec.z,
              });
            }

            Vec::write_options(&truncated_vertices, writer, endian, ())?;
            offset_in_data += 16 * self.object.vertex_count as usize;
          } else {
            cursor.set_position(offset_in_data as u64);
            global_vertices = Vec::<Vector3>::read_options(
              &mut cursor,
              binrw::Endian::Big,
              binrw::VecArgs::builder()
                .count(self.object.vertex_count as usize)
                .finalize(),
            )
            .unwrap();

            Vec::write_options(&global_vertices, writer, endian, ())?;
            offset_in_data += 12 * self.object.vertex_count as usize;

            // let data = (&args.streaming_data[offset_in_data..offset_in_data + 12 * self.object.vertex_count as usize]).to_vec();
            // Vec::write_options(&data, writer, options, ())?;
            // offset_in_data += 12 * self.object.vertex_count as usize;
          }
        }
        if self.object.quantized == 1 {
          if args.ror {
            // Read Vector4i16s from the data and write out Vector3i16s, truncating the W component.
            cursor.set_position(offset_in_data as u64);
            let vertices = Vec::<Vector4i16>::read_options(
              &mut cursor,
              Endian::Big,
              binrw::VecArgs::builder()
                .count(self.object.vertex_count as usize)
                .finalize(),
            )
            .unwrap();

            let mut truncated_vertices = Vec::new();
            for vec in vertices.iter() {
              truncated_vertices.push(Vector3i16 {
                x: vec.x,
                y: vec.y,
                z: vec.z,
              });
            }

            Vec::write_options(&truncated_vertices, writer, endian, ())?;
            offset_in_data += 8 * self.object.vertex_count as usize;
          } else {
            cursor.set_position(offset_in_data as u64);
            let quantized_vertices = Vec::<Vector3i16>::read_options(
              &mut cursor,
              binrw::Endian::Big,
              binrw::VecArgs::builder()
                .count(self.object.vertex_count as usize)
                .finalize(),
            )
            .unwrap();

            Vec::write_options(&quantized_vertices, writer, endian, ())?;
            offset_in_data += 6 * self.object.vertex_count as usize;

            for vec in quantized_vertices.iter() {
              global_vertices.push(Vector3 {
                x: vec.x as f32 / 16384.0_f32,
                y: vec.y as f32 / 16384.0_f32,
                z: vec.z as f32 / 16384.0_f32,
              });
            }

            // let data = (&args.streaming_data[offset_in_data..offset_in_data + 6 * self.object.vertex_count as usize]).to_vec();
            // Vec::write_options(&data, writer, options, ())?;
            // offset_in_data += 6 * self.object.vertex_count as usize;
          }
        }
        i32::write_options(&self.object.load_normals, writer, endian, ())?;
        if self.object.load_normals == 1 {
          if self.object.quantized == 0 {
            if args.ror {
              // Read Vector4s from the data and write out Vector3s, truncating the W component.
              cursor.set_position(offset_in_data as u64);
              let normals = Vec::<Vector4>::read_options(
                &mut cursor,
                binrw::Endian::Big,
                binrw::VecArgs::builder()
                  .count(self.object.vertex_count as usize)
                  .finalize(),
              )
              .unwrap();

              let mut truncated_normals = Vec::new();
              for vec in normals.iter() {
                truncated_normals.push(Vector3 {
                  x: vec.x,
                  y: vec.y,
                  z: vec.z,
                });
              }

              Vec::write_options(&truncated_normals, writer, endian, ())?;
              offset_in_data += 16 * self.object.vertex_count as usize;
            } else {
              let data = (&args.streaming_data
                [offset_in_data..offset_in_data + 12 * self.object.vertex_count as usize])
                .to_vec();
              Vec::write_options(&data, writer, endian, ())?;
              offset_in_data += 12 * self.object.vertex_count as usize;
            }
          }
          if self.object.quantized == 1 {
            if args.ror {
              // Read Vector4i16s from the data and write out Vector3i16s, truncating the W component.
              cursor.set_position(offset_in_data as u64);
              let normals = Vec::<Vector4i16>::read_options(
                &mut cursor,
                binrw::Endian::Big,
                binrw::VecArgs::builder()
                  .count(self.object.vertex_count as usize)
                  .finalize(),
              )
              .unwrap();

              let mut truncated_normals = Vec::new();
              for vec in normals.iter() {
                truncated_normals.push(Vector3i16 {
                  x: vec.x,
                  y: vec.y,
                  z: vec.z,
                });
              }

              Vec::write_options(&truncated_normals, writer, endian, ())?;
              offset_in_data += 8 * self.object.vertex_count as usize;
            } else {
              let data = (&args.streaming_data
                [offset_in_data..offset_in_data + 6 * self.object.vertex_count as usize])
                .to_vec();
              Vec::write_options(&data, writer, endian, ())?;
              offset_in_data += 6 * self.object.vertex_count as usize;
            }
          }
        }
        // Nasty hack...
        if args.ror && self.object.vertex_count % 2 == 1 {
          offset_in_data += 16;
        }
        if args.ror {
          // Read RaceORamaStreamingDataTreeFace from the data and write out TreeFaces.
          cursor.set_position(offset_in_data as u64);
          let ror_tree_faces = Vec::<RaceORamaStreamingDataTreeFace>::read_options(
            &mut cursor,
            binrw::Endian::Big,
            binrw::VecArgs::builder()
              .count(self.object.tree_face_count as usize)
              .finalize(),
          )
          .unwrap();

          let mut tree_faces = Vec::new();
          for vec in ror_tree_faces.iter() {
            tree_faces.push(vec.to_tree_face());
          }

          Vec::write_options(&tree_faces, writer, endian, ())?;
          offset_in_data += 64 * self.object.tree_face_count as usize;
        } else {
          // Read StreamingDataTreeFaces from the data and write out TreeFaces.
          cursor.set_position(offset_in_data as u64);
          let mn_tree_faces = Vec::<StreamingDataTreeFace>::read_options(
            &mut cursor,
            binrw::Endian::Big,
            binrw::VecArgs::builder()
              .count(self.object.tree_face_count as usize)
              .finalize(),
          )
          .unwrap();

          let mut tree_faces = Vec::new();
          for vec in mn_tree_faces.iter() {
            tree_faces.push(vec.to_tree_face());
          }

          Vec::write_options(&tree_faces, writer, endian, ())?;
          offset_in_data += 36 * self.object.tree_face_count as usize;
        }
        if args.ror {
          // Read StreamingDataTreeFaceLeaves from the data and write out TreeFaceLeaves.
          cursor.set_position(offset_in_data as u64);
          let ror_face_leaves = Vec::<RaceORamaStreamingDataTreeFaceLeaf>::read_options(
            &mut cursor,
            binrw::Endian::Big,
            binrw::VecArgs::builder()
              .count(self.object.tree_face_leaf_count as usize)
              .finalize(),
          )
          .unwrap();

          let mut tree_face_leaves = Vec::new();
          for vec in ror_face_leaves.iter() {
            tree_face_leaves.push(vec.to_tree_face_leaf());
          }

          Vec::write_options(&tree_face_leaves, writer, endian, ())?;
          offset_in_data += 32 * self.object.tree_face_leaf_count as usize;
        } else {
          // Read StreamingDataTreeFaceLeaves from the data and write out TreeFaceLeaves.
          cursor.set_position(offset_in_data as u64);
          let mn_face_leaves = Vec::<StreamingDataTreeFaceLeaf>::read_options(
            &mut cursor,
            binrw::Endian::Big,
            binrw::VecArgs::builder()
              .count(self.object.tree_face_leaf_count as usize)
              .finalize(),
          )
          .unwrap();

          let mut tree_face_leaves = Vec::new();
          for vec in mn_face_leaves.iter() {
            tree_face_leaves.push(vec.to_tree_face_leaf(&global_vertices));
          }

          Vec::write_options(&tree_face_leaves, writer, endian, ())?;
          offset_in_data += 20 * self.object.tree_face_leaf_count as usize;
        }
        assert_eq!(offset_in_data, args.streaming_data.len());
      }
      CollisionType::StreamingHeirarchy => {
        CollisionType::write_options(&CollisionType::SoultreeHeirarchy, writer, endian, ())?;

        i32::write_options(&self.object_count, writer, endian, ())?;
        i32::write_options(&self.reverse_collision_mode, writer, endian, ())?;
        for object in &self.objects {
          i32::write_options(&object.object.temp_cmt, writer, endian, ())?;
          object.object.obb_data.write_options(writer, endian, ())?;
          i32::write_options(&object.object.reverse_collision_mode, writer, endian, ())?;
          u32::write_options(&object.object.vertex_count, writer, endian, ())?;
          u32::write_options(&object.object.tree_face_count, writer, endian, ())?;
          u32::write_options(&object.object.tree_face_leaf_count, writer, endian, ())?;
          i32::write_options(&object.object.quantized, writer, endian, ())?;

          // Conversion from MN's streaming format requires the vertices be cached and used later for DValue calculation.
          let mut global_vertices = Vec::new();

          if object.object.quantized == 0 {
            if args.ror {
              // Read Vector4s from the data and write out Vector3s, truncating the W component.
              cursor.set_position(offset_in_data as u64);
              let vertices = Vec::<Vector4>::read_options(
                &mut cursor,
                binrw::Endian::Big,
                binrw::VecArgs::builder()
                  .count(object.object.vertex_count as usize)
                  .finalize(),
              )
              .unwrap();

              let mut truncated_vertices = Vec::new();
              for vec in vertices.iter() {
                truncated_vertices.push(Vector3 {
                  x: vec.x,
                  y: vec.y,
                  z: vec.z,
                });
              }

              Vec::write_options(&truncated_vertices, writer, endian, ())?;
              offset_in_data += 16 * object.object.vertex_count as usize;
            } else {
              cursor.set_position(offset_in_data as u64);
              global_vertices = Vec::<Vector3>::read_options(
                &mut cursor,
                binrw::Endian::Big,
                binrw::VecArgs::builder()
                  .count(object.object.vertex_count as usize)
                  .finalize(),
              )
              .unwrap();

              Vec::write_options(&global_vertices, writer, endian, ())?;
              offset_in_data += 12 * object.object.vertex_count as usize;

              // let data = (&args.streaming_data[offset_in_data..offset_in_data + 12 * object.object.vertex_count as usize]).to_vec();
              // Vec::write_options(&data, writer, options, ())?;
              // offset_in_data += 12 * object.object.vertex_count as usize;
            }
          }
          if object.object.quantized == 1 {
            if args.ror {
              // Read Vector4i16s from the data and write out Vector3i16s, truncating the W component.
              cursor.set_position(offset_in_data as u64);
              let vertices = Vec::<Vector4i16>::read_options(
                &mut cursor,
                binrw::Endian::Big,
                binrw::VecArgs::builder()
                  .count(object.object.vertex_count as usize)
                  .finalize(),
              )
              .unwrap();

              let mut truncated_vertices = Vec::new();
              for vec in vertices.iter() {
                truncated_vertices.push(Vector3i16 {
                  x: vec.x,
                  y: vec.y,
                  z: vec.z,
                });
              }

              Vec::write_options(&truncated_vertices, writer, endian, ())?;
              offset_in_data += 8 * object.object.vertex_count as usize;
            } else {
              cursor.set_position(offset_in_data as u64);
              let quantized_vertices = Vec::<Vector3i16>::read_options(
                &mut cursor,
                binrw::Endian::Big,
                binrw::VecArgs::builder()
                  .count(object.object.vertex_count as usize)
                  .finalize(),
              )
              .unwrap();

              Vec::write_options(&quantized_vertices, writer, endian, ())?;
              offset_in_data += 6 * object.object.vertex_count as usize;

              for vec in quantized_vertices.iter() {
                global_vertices.push(Vector3 {
                  x: vec.x as f32 / 16384.0_f32,
                  y: vec.y as f32 / 16384.0_f32,
                  z: vec.z as f32 / 16384.0_f32,
                });
              }

              // let data = (&args.streaming_data[offset_in_data..offset_in_data + 6 * object.object.vertex_count as usize]).to_vec();
              // Vec::write_options(&data, writer, options, ())?;
              // offset_in_data += 6 * object.object.vertex_count as usize;
            }
          }
          i32::write_options(&object.object.load_normals, writer, endian, ())?;
          if object.object.load_normals == 1 {
            if object.object.quantized == 0 {
              if args.ror {
                // Read Vector4s from the data and write out Vector3s, truncating the W component.
                cursor.set_position(offset_in_data as u64);
                let normals = Vec::<Vector4>::read_options(
                  &mut cursor,
                  binrw::Endian::Big,
                  binrw::VecArgs::builder()
                    .count(object.object.vertex_count as usize)
                    .finalize(),
                )
                .unwrap();

                let mut truncated_normals = Vec::new();
                for vec in normals.iter() {
                  truncated_normals.push(Vector3 {
                    x: vec.x,
                    y: vec.y,
                    z: vec.z,
                  });
                }

                Vec::write_options(&truncated_normals, writer, endian, ())?;
                offset_in_data += 16 * object.object.vertex_count as usize;
              } else {
                let data = (&args.streaming_data
                  [offset_in_data..offset_in_data + 12 * object.object.vertex_count as usize])
                  .to_vec();
                Vec::write_options(&data, writer, endian, ())?;
                offset_in_data += 12 * object.object.vertex_count as usize;
              }
            }
            if object.object.quantized == 1 {
              if args.ror {
                // Read Vector4i16s from the data and write out Vector3i16s, truncating the W component.
                cursor.set_position(offset_in_data as u64);
                let normals = Vec::<Vector4i16>::read_options(
                  &mut cursor,
                  binrw::Endian::Big,
                  binrw::VecArgs::builder()
                    .count(object.object.vertex_count as usize)
                    .finalize(),
                )
                .unwrap();

                let mut truncated_normals = Vec::new();
                for vec in normals.iter() {
                  truncated_normals.push(Vector3i16 {
                    x: vec.x,
                    y: vec.y,
                    z: vec.z,
                  });
                }

                Vec::write_options(&truncated_normals, writer, endian, ())?;
                offset_in_data += 8 * object.object.vertex_count as usize;
              } else {
                let data = (&args.streaming_data
                  [offset_in_data..offset_in_data + 6 * object.object.vertex_count as usize])
                  .to_vec();
                Vec::write_options(&data, writer, endian, ())?;
                offset_in_data += 6 * object.object.vertex_count as usize;
              }
            }
          }
          // Nasty hack...
          if args.ror && object.object.vertex_count % 2 == 1 {
            offset_in_data += 16;
          }
          if args.ror {
            // Read RaceORamaStreamingDataTreeFace from the data and write out TreeFaces.
            cursor.set_position(offset_in_data as u64);
            let ror_tree_faces = Vec::<RaceORamaStreamingDataTreeFace>::read_options(
              &mut cursor,
              binrw::Endian::Big,
              binrw::VecArgs::builder()
                .count(object.object.tree_face_count as usize)
                .finalize(),
            )
            .unwrap();

            let mut tree_faces = Vec::new();
            for vec in ror_tree_faces.iter() {
              tree_faces.push(vec.to_tree_face());
            }

            Vec::write_options(&tree_faces, writer, endian, ())?;
            offset_in_data += 64 * object.object.tree_face_count as usize;
          } else {
            // Read StreamingDataTreeFaces from the data and write out TreeFaces.
            cursor.set_position(offset_in_data as u64);
            let mn_tree_faces = Vec::<StreamingDataTreeFace>::read_options(
              &mut cursor,
              binrw::Endian::Big,
              binrw::VecArgs::builder()
                .count(object.object.tree_face_count as usize)
                .finalize(),
            )
            .unwrap();

            let mut tree_faces = Vec::new();
            for vec in mn_tree_faces.iter() {
              tree_faces.push(vec.to_tree_face());
            }

            Vec::write_options(&tree_faces, writer, endian, ())?;
            offset_in_data += 36 * object.object.tree_face_count as usize;
          }
          if args.ror {
            // Read StreamingDataTreeFaceLeaves from the data and write out TreeFaceLeaves.
            cursor.set_position(offset_in_data as u64);
            let ror_face_leaves = Vec::<RaceORamaStreamingDataTreeFaceLeaf>::read_options(
              &mut cursor,
              binrw::Endian::Big,
              binrw::VecArgs::builder()
                .count(object.object.tree_face_leaf_count as usize)
                .finalize(),
            )
            .unwrap();

            let mut tree_face_leaves = Vec::new();
            for vec in ror_face_leaves.iter() {
              tree_face_leaves.push(vec.to_tree_face_leaf());
            }

            Vec::write_options(&tree_face_leaves, writer, endian, ())?;
            offset_in_data += 32 * object.object.tree_face_leaf_count as usize;
          } else {
            // Read StreamingDataTreeFaceLeaves from the data and write out TreeFaceLeaves.
            cursor.set_position(offset_in_data as u64);
            let mn_face_leaves = Vec::<StreamingDataTreeFaceLeaf>::read_options(
              &mut cursor,
              binrw::Endian::Big,
              binrw::VecArgs::builder()
                .count(object.object.tree_face_leaf_count as usize)
                .finalize(),
            )
            .unwrap();

            let mut tree_face_leaves = Vec::new();
            for vec in mn_face_leaves.iter() {
              tree_face_leaves.push(vec.to_tree_face_leaf(&global_vertices));
            }

            Vec::write_options(&tree_face_leaves, writer, endian, ())?;
            offset_in_data += 20 * object.object.tree_face_leaf_count as usize;
          }
          assert_eq!(offset_in_data, args.streaming_data.len());
        }
      }
      CollisionType::StreamingFinitePlane => {
        CollisionType::write_options(&CollisionType::FinitePlane, writer, endian, ())?;

        i32::write_options(&self.plane_count, writer, endian, ())?;
        Vector3::write_options(&self.half, writer, endian, ())?;

        Vec::write_options(&args.streaming_data, writer, endian, ())?;
        // assert_eq!(self.plane_count as usize * 60, args.streaming_data.len());
      }
    }
    Ok(())
  }
}
