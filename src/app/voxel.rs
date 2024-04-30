#[derive(Debug, Clone, Copy)]
pub struct Voxel {
    pub faces: [Face; 3],
}

#[derive(Debug, Clone, Copy)]
pub struct Face {
    pub color: [f32; 4],
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum VoxelSide {
    Top,
    Side,
    Bottom,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}
