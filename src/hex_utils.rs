use crate::types::HexCoord;
use bevy::prelude::*;

pub struct HexGeometry;

impl HexGeometry {
    pub const SIZE: f32 = 1.0;
    pub const SQRT3: f32 = 1.7320508;
    pub const SQRT3_2: f32 = 0.8660254;

    pub fn hex_to_world(hex: &HexCoord) -> Vec3 {
        let x = Self::SIZE * (Self::SQRT3 * hex.q as f32 + Self::SQRT3_2 * hex.r as f32);
        let z = Self::SIZE * (1.5 * hex.r as f32);
        Vec3::new(x, 0.0, z)
    }

    pub fn world_to_hex(pos: &Vec3) -> HexCoord {
        let q = (Self::SQRT3 / 3.0 * pos.x - 1.0 / 3.0 * pos.z) / Self::SIZE;
        let r = (2.0 / 3.0 * pos.z) / Self::SIZE;

        let q_round = q.round() as i32;
        let r_round = r.round() as i32;

        HexCoord::new(q_round, r_round)
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
