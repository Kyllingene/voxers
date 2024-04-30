use super::voxel::{Face, Voxel};

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Block {
    #[default]
    Air,
    Water,
    Grass,
    Dirt,
    Stone,
}

impl Block {
    pub fn voxel(self) -> Voxel {
        VOXELS[self as usize]
    }

    #[allow(clippy::match_like_matches_macro)]
    pub const fn is_transparent(self) -> bool {
        match self {
            Block::Air | Block::Water => true,
            _ => false,
        }
    }
}

pub const VOXELS: &[Voxel] = &[
    Voxel {
        // Air
        faces: [Face { color: [0.0; 4] }; 3],
    },
    Voxel {
        // Water
        faces: [Face {
            color: [0.0, 0.3, 0.7, 0.2],
        }; 3],
    },
    Voxel {
        // Grass
        faces: [
            Face {
                color: [0.22, 0.56, 0.24, 1.0],
            },
            Face {
                color: [0.529, 0.243, 0.137, 1.0],
            },
            Face {
                color: [0.529, 0.243, 0.137, 1.0],
            },
        ],
    },
    Voxel {
        // Dirt
        faces: [Face {
            color: [0.529, 0.243, 0.137, 1.0],
        }; 3],
    },
    Voxel {
        // Stone
        faces: [Face {
            color: [0.62, 0.62, 0.62, 1.0],
        }; 3],
    },
];
