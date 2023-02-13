use binrw::{BinRead, BinWrite};

#[derive(BinRead, BinWrite, Debug)]
#[br(big)]
#[bw(big)]
struct MotionPackString {
  len: u32,

  #[br(count = len)]
  bone_name: Vec<u8>,
}

#[derive(BinRead, BinWrite, Debug)]
#[br(big)]
#[bw(big)]
pub struct StreamingMotionPackHeader {
  version: i32,
  motion_type: i32,
  frame_count: u32,
  object_count: u32,

  #[br(count = object_count)]
  bone_targets: Vec<MotionPackString>,

  duration: f32,
  rotation_type: u32,
  position_type: u32,
  num_positions: u32,
  num_rotations: u32,
  num_camera_infos: u32,
  padding: u32,

  #[br(if (version == - 8))]
  #[bw(ignore)]
  unk_bpb_size: u32,

  #[br(if (version == - 8), count = unk_bpb_size)]
  #[bw(ignore)]
  unk_bpb: Vec<u8>,
}

#[derive(BinRead, Debug)]
pub struct StreamingMotionPack {
  pub model_info: crate::ModelInfo,
  pub header: StreamingMotionPackHeader,
}
