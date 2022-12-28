use crate::model::{Vector3, Vector4};
use binrw::{BinRead, BinReaderExt, BinResult, BinWrite, ReadOptions};

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

#[derive(BinRead, BinWrite, Debug)]
#[brw(big)]
struct Vector3i16 {
  pub x: i16,
  pub y: i16,
  pub z: i16,
}

#[derive(BinRead, BinWrite, Debug)]
#[brw(big)]
struct Vector4i16 {
  pub x: i16,
  pub y: i16,
  pub z: i16,
  pub w: i16,
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
struct StreamingDataTreeFace {
  vectors: [Vector4; 2],
  type_indices: [i16; 2],
  volume: f32,
  #[brw(pad_after = 20)]
  radius: f32,
}

impl StreamingDataTreeFace {
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

#[derive(BinRead, BinWrite, Debug)]
#[brw(big)]
struct TreeFaceLeaf {
  dvalue: f32,
  vector: Vector3,
  vertices: [i16; 3],
}

#[derive(BinRead, BinWrite, Debug)]
#[brw(big)]
struct StreamingDataTreeFaceLeaf {
  vector: Vector4,
  dvalue: f32,
  #[brw(pad_after = 6)]
  vertices: [i16; 3],
}

impl StreamingDataTreeFaceLeaf {
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

  #[br(if(collision_type == CollisionType::StreamingSoultree))]
  object: SoultreeCollisionObject,

  #[br(if(collision_type == CollisionType::StreamingHeirarchy))]
  object_count: i32,
  #[br(if(collision_type == CollisionType::StreamingHeirarchy))]
  reverse_collision_mode: i32,
  #[br(if(collision_type == CollisionType::StreamingHeirarchy), count = object_count)]
  objects: Vec<StreamingHeirarchyEntry>,

  #[br(if(collision_type == CollisionType::StreamingFinitePlane))]
  plane_count: i32,
  #[br(if(collision_type == CollisionType::StreamingFinitePlane))]
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

#[derive(binrw::BinrwNamedArgs, Clone, Debug)]
pub struct CollisionModelArgs {
  pub streaming_data: Vec<u8>,
}

// This BinWrite implementation actually restructures the streaming component data plus the header data in the SOI to form a proper GOL file.
// As such, the streaming data must be passed to write_options.
impl BinWrite for CollisionModel {
  type Args = CollisionModelArgs;

  fn write_options<W: std::io::Write + std::io::Seek>(
    &self,
    writer: &mut W,
    options: &binrw::WriteOptions,
    args: Self::Args,
  ) -> binrw::BinResult<()> {
    let magic = b"\x00\x00\x04\xD2".to_vec();
    Vec::write_options(&magic, writer, options, ())?;
    self.col_type.write_options(writer, options, ())?;
    i32::write_options(&self.version, writer, options, ())?;

    let mut offset_in_data: usize = 0;

    let mut cursor = std::io::Cursor::new(&args.streaming_data);

    println!("{}", self.collision_type);
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
        CollisionType::write_options(&CollisionType::Soultree, writer, options, ())?;

        i32::write_options(&self.object.temp_cmt, writer, options, ())?;
        self.object.obb_data.write_options(writer, options, ())?;
        i32::write_options(&self.object.reverse_collision_mode, writer, options, ())?;
        u32::write_options(&self.object.vertex_count, writer, options, ())?;
        u32::write_options(&self.object.tree_face_count, writer, options, ())?;
        u32::write_options(&self.object.tree_face_leaf_count, writer, options, ())?;
        i32::write_options(&self.object.quantized, writer, options, ())?;

        if self.object.quantized == 0 {
          // Read Vector4s from the data and write out Vector3s, truncating the W component.
          cursor.set_position(offset_in_data as u64);
          let vectors = Vec::<Vector4>::read_options(
            &mut cursor,
            &ReadOptions::new(binrw::Endian::Big),
            binrw::VecArgs::builder()
              .count(self.object.vertex_count as usize)
              .finalize(),
          )
          .unwrap();

          let mut truncated_vectors = Vec::new();
          for vec in vectors.iter() {
            truncated_vectors.push(Vector3 {
              x: vec.x,
              y: vec.y,
              z: vec.z,
            });
          }

          Vec::write_options(&truncated_vectors, writer, options, ())?;
          offset_in_data += 16 * self.object.vertex_count as usize;
        }
        if self.object.quantized == 1 {
          // Read Vector4i16s from the data and write out Vector3i16s, truncating the W component.
          cursor.set_position(offset_in_data as u64);
          let vectors = Vec::<Vector4i16>::read_options(
            &mut cursor,
            &ReadOptions::new(binrw::Endian::Big),
            binrw::VecArgs::builder()
              .count(self.object.vertex_count as usize)
              .finalize(),
          )
          .unwrap();

          let mut truncated_vectors = Vec::new();
          for vec in vectors.iter() {
            truncated_vectors.push(Vector3i16 {
              x: vec.x,
              y: vec.y,
              z: vec.z,
            });
          }

          Vec::write_options(&truncated_vectors, writer, options, ())?;
          offset_in_data += 8 * self.object.vertex_count as usize;
        }
        i32::write_options(&self.object.load_normals, writer, options, ())?;
        if self.object.load_normals == 1 {
          if self.object.quantized == 0 {
            // Read Vector4s from the data and write out Vector3s, truncating the W component.
            cursor.set_position(offset_in_data as u64);
            let vectors = Vec::<Vector4>::read_options(
              &mut cursor,
              &ReadOptions::new(binrw::Endian::Big),
              binrw::VecArgs::builder()
                .count(self.object.vertex_count as usize)
                .finalize(),
            )
            .unwrap();

            let mut truncated_vectors = Vec::new();
            for vec in vectors.iter() {
              truncated_vectors.push(Vector3 {
                x: vec.x,
                y: vec.y,
                z: vec.z,
              });
            }

            Vec::write_options(&truncated_vectors, writer, options, ())?;
            offset_in_data += 16 * self.object.vertex_count as usize;
          }
          if self.object.quantized == 1 {
            // Read Vector4i16s from the data and write out Vector3i16s, truncating the W component.
            cursor.set_position(offset_in_data as u64);
            let vectors = Vec::<Vector4i16>::read_options(
              &mut cursor,
              &ReadOptions::new(binrw::Endian::Big),
              binrw::VecArgs::builder()
                .count(self.object.vertex_count as usize)
                .finalize(),
            )
            .unwrap();

            let mut truncated_vectors = Vec::new();
            for vec in vectors.iter() {
              truncated_vectors.push(Vector3i16 {
                x: vec.x,
                y: vec.y,
                z: vec.z,
              });
            }

            Vec::write_options(&truncated_vectors, writer, options, ())?;
            offset_in_data += 8 * self.object.vertex_count as usize;
          }
        }
        // Nasty hack...
        if self.object.vertex_count % 2 == 1 {
          offset_in_data += 16;
        }

        // Read StreamingDataTreeFaces from the data and write out TreeFaces.
        cursor.set_position(offset_in_data as u64);
        let vectors = Vec::<StreamingDataTreeFace>::read_options(
          &mut cursor,
          &ReadOptions::new(binrw::Endian::Big),
          binrw::VecArgs::builder()
            .count(self.object.tree_face_count as usize)
            .finalize(),
        )
        .unwrap();

        let mut tree_faces = Vec::new();
        for vec in vectors.iter() {
          tree_faces.push(vec.to_tree_face());
        }

        Vec::write_options(&tree_faces, writer, options, ())?;
        offset_in_data += 64 * self.object.tree_face_count as usize;

        // Read StreamingDataTreeFaceLeaves from the data and write out TreeFaceLeaves.
        cursor.set_position(offset_in_data as u64);
        let vectors = Vec::<StreamingDataTreeFaceLeaf>::read_options(
          &mut cursor,
          &ReadOptions::new(binrw::Endian::Big),
          binrw::VecArgs::builder()
            .count(self.object.tree_face_leaf_count as usize)
            .finalize(),
        )
        .unwrap();

        let mut tree_face_leaves = Vec::new();
        for vec in vectors.iter() {
          tree_face_leaves.push(vec.to_tree_face_leaf());
        }

        Vec::write_options(&tree_face_leaves, writer, options, ())?;
        offset_in_data += 32 * self.object.tree_face_leaf_count as usize;
        // assert_eq!(offset_in_data, args.streaming_data.len());
      }
      CollisionType::StreamingHeirarchy => {
        CollisionType::write_options(&CollisionType::SoultreeHeirarchy, writer, options, ())?;

        i32::write_options(&self.object_count, writer, options, ())?;
        i32::write_options(&self.reverse_collision_mode, writer, options, ())?;
        for object in &self.objects {
          i32::write_options(&object.object.temp_cmt, writer, options, ())?;
          object.object.obb_data.write_options(writer, options, ())?;
          i32::write_options(&object.object.reverse_collision_mode, writer, options, ())?;
          u32::write_options(&object.object.vertex_count, writer, options, ())?;
          u32::write_options(&object.object.tree_face_count, writer, options, ())?;
          u32::write_options(&object.object.tree_face_leaf_count, writer, options, ())?;
          i32::write_options(&object.object.quantized, writer, options, ())?;

          if object.object.quantized == 0 {
            // Read Vector4s from the data and write out Vector3s, truncating the W component.
            cursor.set_position(offset_in_data as u64);
            let vectors = Vec::<Vector4>::read_options(
              &mut cursor,
              &ReadOptions::new(binrw::Endian::Big),
              binrw::VecArgs::builder()
                .count(object.object.vertex_count as usize)
                .finalize(),
            )
            .unwrap();

            let mut truncated_vectors = Vec::new();
            for vec in vectors.iter() {
              truncated_vectors.push(Vector3 {
                x: vec.x,
                y: vec.y,
                z: vec.z,
              });
            }

            Vec::write_options(&truncated_vectors, writer, options, ())?;
            offset_in_data += 16 * object.object.vertex_count as usize;
          }
          if object.object.quantized == 1 {
            // Read Vector4i16s from the data and write out Vector3i16s, truncating the W component.
            cursor.set_position(offset_in_data as u64);
            let vectors = Vec::<Vector4i16>::read_options(
              &mut cursor,
              &ReadOptions::new(binrw::Endian::Big),
              binrw::VecArgs::builder()
                .count(object.object.vertex_count as usize)
                .finalize(),
            )
            .unwrap();

            let mut truncated_vectors = Vec::new();
            for vec in vectors.iter() {
              truncated_vectors.push(Vector3i16 {
                x: vec.x,
                y: vec.y,
                z: vec.z,
              });
            }

            Vec::write_options(&truncated_vectors, writer, options, ())?;
            offset_in_data += 8 * object.object.vertex_count as usize;
          }
          i32::write_options(&object.object.load_normals, writer, options, ())?;
          if object.object.load_normals == 1 {
            if object.object.quantized == 0 {
              // Read Vector4s from the data and write out Vector3s, truncating the W component.
              cursor.set_position(offset_in_data as u64);
              let vectors = Vec::<Vector4>::read_options(
                &mut cursor,
                &ReadOptions::new(binrw::Endian::Big),
                binrw::VecArgs::builder()
                  .count(object.object.vertex_count as usize)
                  .finalize(),
              )
              .unwrap();

              let mut truncated_vectors = Vec::new();
              for vec in vectors.iter() {
                truncated_vectors.push(Vector3 {
                  x: vec.x,
                  y: vec.y,
                  z: vec.z,
                });
              }

              Vec::write_options(&truncated_vectors, writer, options, ())?;
              offset_in_data += 16 * object.object.vertex_count as usize;
            }
            if object.object.quantized == 1 {
              // Read Vector4i16s from the data and write out Vector3i16s, truncating the W component.
              cursor.set_position(offset_in_data as u64);
              let vectors = Vec::<Vector4i16>::read_options(
                &mut cursor,
                &ReadOptions::new(binrw::Endian::Big),
                binrw::VecArgs::builder()
                  .count(object.object.vertex_count as usize)
                  .finalize(),
              )
              .unwrap();

              let mut truncated_vectors = Vec::new();
              for vec in vectors.iter() {
                truncated_vectors.push(Vector3i16 {
                  x: vec.x,
                  y: vec.y,
                  z: vec.z,
                });
              }

              Vec::write_options(&truncated_vectors, writer, options, ())?;
              offset_in_data += 8 * object.object.vertex_count as usize;
            }
          }
          // Nasty hack...
          if object.object.vertex_count % 2 == 1 {
            offset_in_data += 16;
          }

          // Read StreamingDataTreeFaces from the data and write out TreeFaces.
          cursor.set_position(offset_in_data as u64);
          let vectors = Vec::<StreamingDataTreeFace>::read_options(
            &mut cursor,
            &ReadOptions::new(binrw::Endian::Big),
            binrw::VecArgs::builder()
              .count(object.object.tree_face_count as usize)
              .finalize(),
          )
          .unwrap();

          let mut tree_faces = Vec::new();
          for vec in vectors.iter() {
            tree_faces.push(vec.to_tree_face());
          }

          Vec::write_options(&tree_faces, writer, options, ())?;
          offset_in_data += 64 * object.object.tree_face_count as usize;

          // Read StreamingDataTreeFaceLeaves from the data and write out TreeFaceLeaves.
          cursor.set_position(offset_in_data as u64);
          let vectors = Vec::<StreamingDataTreeFaceLeaf>::read_options(
            &mut cursor,
            &ReadOptions::new(binrw::Endian::Big),
            binrw::VecArgs::builder()
              .count(object.object.tree_face_leaf_count as usize)
              .finalize(),
          )
          .unwrap();

          let mut tree_face_leaves = Vec::new();
          for vec in vectors.iter() {
            tree_face_leaves.push(vec.to_tree_face_leaf());
          }

          Vec::write_options(&tree_face_leaves, writer, options, ())?;
          offset_in_data += 32 * object.object.tree_face_leaf_count as usize;
          // assert_eq!(offset_in_data, args.streaming_data.len());
        }
        assert_eq!(offset_in_data, args.streaming_data.len());
      }
      CollisionType::StreamingFinitePlane => {
        CollisionType::write_options(&CollisionType::FinitePlane, writer, options, ())?;

        i32::write_options(&self.plane_count, writer, options, ())?;
        Vector3::write_options(&self.half, writer, options, ())?;

        Vec::write_options(&args.streaming_data, writer, options, ())?;
        // assert_eq!(self.plane_count as usize * 60, args.streaming_data.len());
      }
    }
    Ok(())
  }
}
