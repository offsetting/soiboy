use crate::model::{Vector3, Vector4};
use binrw::{BinRead, BinReaderExt, BinResult, BinWrite};

// Members that are commented out are part of the streaming data section, and need to be merged into the contents of the header data after extraction.

#[derive(BinRead, BinWrite, PartialEq, Debug)]
#[br(repr = i32)]
#[bw(repr = i32)]
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

#[derive(BinRead, BinWrite, Debug)]
#[br(big)]
#[bw(big)]
struct ShortVector {
  x: i16,
  y: i16,
  z: i16,
}

#[derive(Default, BinRead, BinWrite, Debug)]
#[br(big)]
#[bw(big)]
struct TreeFace {
  volume: f32,
  unk: [Vector3; 2],
  type_indices: [i16; 2],
}

#[derive(BinRead, BinWrite, Debug)]
#[br(big)]
#[bw(big)]
struct TreeFaceLeaf {
  dvalue: f32,
  vector: ShortVector,
  vertices: [i16; 3],
}

#[derive(Default, BinRead, BinWrite, Debug)]
#[br(big)]
#[bw(big)]
struct SoultreeCollisionObject {
  obb_data: [f32; 12],
  reverse_collision_mode: i32,
  vertex_count: u32,
  tree_face_count: u32,
  tree_face_leaf_count: u32,
  quantized: i32,

  // #[br(if(quantized == 1), count = vertex_count)]
  // quantized_tree_vertices: Vec<ShortVector>,

  // #[br(if(quantized == 0), count = vertex_count)]
  // tree_vertices: Vec<Vector3>,
  load_normals: i32,

  // #[br(if(load_normals == 1 && quantized == 1), count = vertex_count)]
  // quantized_tree_normals: Vec<ShortVector>,

  // #[br(if(load_normals == 1 && quantized == 0), count = vertex_count)]
  // tree_normals: Vec<Vector3>,

  // #[br(count = tree_face_count)]
  // tree_faces: Vec<TreeFace>,
  top_tree_face: TreeFace,

  // #[br(count = tree_face_leaf_count)]
  // tree_face_leaves: Vec<TreeFaceLeaf>,
}

#[derive(BinRead, BinWrite, Debug)]
#[br(big)]
#[bw(big)]
struct FinitePlaneStruct {
  local_vertex_bl: Vector3,
  local_vertex_br: Vector3,
  local_vertex_tl: Vector3,
  local_vertex_tr: Vector3,
  plane_normal: Vector3,
}

// todo
// br ignore all the non streaming bits, add them back
// when checking for col type check against non streaming
#[derive(BinRead, BinWrite, Debug)]
#[br(big)]
#[bw(big)]
#[br(magic = b"\x00\x00\x04\xD2")]
pub struct CollisionModel {
  col_type: [u8; 4],
  version: i32,
  collision_type: CollisionType,

  #[br(if(collision_type == CollisionType::StreamingSoultree))]
  temp_cmt: i32,
  #[br(if(collision_type == CollisionType::StreamingSoultree))]
  object: SoultreeCollisionObject,

  #[br(if(collision_type == CollisionType::StreamingHeirarchy))]
  object_count: i32,
  #[br(if(collision_type == CollisionType::StreamingHeirarchy))]
  reverse_collision_mode: i32,
  // #[br(if(collision_type == CollisionType::StreamingSoultree))]
  // temp_cmt_cpy: i32,
  #[br(if(collision_type == CollisionType::StreamingHeirarchy), count = object_count)]
  objects: Vec<SoultreeCollisionObject>,

  #[br(if(collision_type == CollisionType::StreamingFinitePlane))]
  plane_count: i32,
  #[br(if(collision_type == CollisionType::StreamingFinitePlane))]
  half: Vector3,
  // #[br(if(collision_type == CollisionType::FinitePlane), count = plane_count)]
  // planes: Vec<FinitePlaneStruct>,
}

#[derive(BinRead, Debug)]
pub struct StreamingCollisionModel {
  model_info: crate::ModelInfo,

  #[br(count = model_info.parameter_count)]
  pub parameters: Vec<crate::StreamingParameter>,
  
  collision_model: CollisionModel,
}
