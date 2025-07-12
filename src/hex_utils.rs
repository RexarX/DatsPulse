use crate::types::HexCoord;
use bevy::prelude::*;

pub struct HexGeometry;

impl HexGeometry {
    pub const SIZE: f32 = 1.0;
    pub const SQRT3: f32 = 1.7320508;

    pub fn hex_to_world(hex: &HexCoord) -> Vec3 {
        // "odd-r" horizontal layout
        let size = Self::SIZE;
        let width = size * 2.0;
        let height = size * Self::SQRT3;

        let x = size * (3.0 / 2.0 * hex.q as f32);
        let z = height * (hex.r as f32 + 0.5 * (hex.q & 1) as f32);

        Vec3::new(x, 0.0, z)
    }

    pub fn world_to_hex(pos: &Vec3) -> HexCoord {
        let size = Self::SIZE;
        let q = ((2.0 / 3.0) * pos.x / size).round() as i32;
        let r = ((-1.0 / 3.0) * pos.x / size + (Self::SQRT3 / 3.0) * pos.z / size).round() as i32;

        HexCoord::new(q, r)
    }

    pub fn hex_corners(center: Vec3) -> [Vec3; 6] {
        let mut corners = [Vec3::ZERO; 6];
        for i in 0..6 {
            let angle = std::f32::consts::PI / 3.0 * i as f32;
            corners[i] =
                center + Vec3::new(Self::SIZE * angle.cos(), 0.0, Self::SIZE * angle.sin());
        }
        corners
    }
}
