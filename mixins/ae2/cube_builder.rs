// SPDX-License-Identifier: LGPL-3.0-or-later AND Apache-2.0
// CubeBuilder and QuadEmitter.square before vanilla packed-vertex conversion.
// Square coordinate rules: Copyright (c) 2016, 2017, 2018, 2019 FabricMC.
// Modified Rust port; see LICENSES/Fabric-Apache-2.0 and NOTICE.md.
use super::ae2_grid::Direction;
#[derive(Clone, Debug, PartialEq)]
pub struct Texture {
    pub id: String,
    pub rect: [f32; 4],
}
impl Texture {
    pub fn unit(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            rect: [0., 0., 1., 1.],
        }
    }
    fn u(&self, value: f32) -> f32 {
        self.rect[0] + (self.rect[2] - self.rect[0]) * value
    }
    fn v(&self, value: f32) -> f32 {
        self.rect[1] + (self.rect[3] - self.rect[1]) * value
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Quad {
    pub texture: String,
    pub face: Direction,
    pub cull: Option<Direction>,
    pub positions: [[f32; 3]; 4],
    pub uv: [[f32; 2]; 4],
    pub color: u32,
    pub emissive: bool,
}
#[derive(Clone, Debug)]
pub struct CubeBuilder {
    pub output: Vec<Quad>,
    textures: [Option<Texture>; 6],
    pub draw_faces: u8,
    custom_uv: [Option<[f32; 4]>; 6],
    rotations: [usize; 6],
    flip_u: [bool; 6],
    flip_v: [bool; 6],
    pub color: u32,
    pub emissive: bool,
}
impl Default for CubeBuilder {
    fn default() -> Self {
        Self {
            output: vec![],
            textures: std::array::from_fn(|_| None),
            draw_faces: 63,
            custom_uv: [None; 6],
            rotations: [0; 6],
            flip_u: [false; 6],
            flip_v: [false; 6],
            color: u32::MAX,
            emissive: false,
        }
    }
}
impl CubeBuilder {
    pub fn set_texture(&mut self, texture: Texture) {
        self.textures.fill(Some(texture));
    }
    /// Array order is Direction::ALL, unlike Java's up/down/north/south/east/west arguments.
    pub fn set_textures(&mut self, textures: [Texture; 6]) {
        self.textures = textures.map(Some);
    }
    pub fn set_face_texture(&mut self, face: Direction, texture: Texture) {
        self.textures[face as usize] = Some(texture);
    }
    pub fn set_flip_u(&mut self, face: Direction, enabled: bool) {
        self.flip_u[face as usize] = enabled;
    }
    pub fn set_flip_v(&mut self, face: Direction, enabled: bool) {
        self.flip_v[face as usize] = enabled;
    }
    pub fn set_custom_uv(&mut self, face: Direction, uv: [f32; 4]) {
        self.custom_uv[face as usize] = Some(uv);
    }
    pub fn set_rotation(&mut self, face: Direction, rotation: usize) -> Result<(), &'static str> {
        if rotation > 3 {
            return Err("rotation");
        }
        self.rotations[face as usize] = rotation;
        Ok(())
    }
    pub fn set_rgb(&mut self, rgb: u32) {
        self.color = rgb | 0xff000000;
    }
    pub fn set_rgb_floats(&mut self, r: f32, g: f32, b: f32) {
        self.set_rgb(
            ((r * 255.) as i32 as u32) << 16
                | ((g * 255.) as i32 as u32) << 8
                | ((b * 255.) as i32 as u32),
        );
    }
    pub fn add_cube(&mut self, bounds: [f32; 6]) {
        let bounds = bounds.map(|v| v / 16.0);
        for face in Direction::ALL {
            if self.draw_faces & (1 << face as u8) != 0 {
                self.add_quad(face, bounds);
            }
        }
    }
    /// addQuad uses already normalized coordinates; addCube uses pixels.
    pub fn add_quad(&mut self, face: Direction, [x1, y1, z1, x2, y2, z2]: [f32; 6]) {
        let i = face as usize;
        let texture = self.textures[i].as_ref().expect("missing face texture");
        let [mut u1, mut v1, mut u2, mut v2] = if let Some(uv) = self.custom_uv[i] {
            uv
        } else {
            let [v1, v2] = if matches!(face, Direction::Down | Direction::Up) {
                [z1, z2]
            } else {
                [1. - y1, 1. - y2]
            };
            let [u1, u2] = match face {
                Direction::Down | Direction::Up | Direction::South => [x1, x2],
                Direction::North => [1. - x2, 1. - x1],
                Direction::West => [z1, z2],
                Direction::East => [1. - z2, 1. - z1],
            };
            [u1, v1, u2, v2]
        };
        u1 = texture.u(u1);
        u2 = texture.u(u2);
        v1 = texture.v(v1);
        v2 = texture.v(v2);
        if self.flip_u[i] {
            std::mem::swap(&mut u1, &mut u2);
        }
        if self.flip_v[i] {
            std::mem::swap(&mut v1, &mut v2);
        }
        let values = if matches!(face, Direction::Down | Direction::Up) {
            [[u1, v1], [u1, v2], [u2, v2], [u2, v1]]
        } else {
            [[u1, v2], [u1, v1], [u2, v1], [u2, v2]]
        };
        let mut uv = [[0.; 2]; 4];
        for (n, value) in values.into_iter().enumerate() {
            uv[(n + 4 - self.rotations[i]) % 4] = value;
        }
        let [mut left, mut bottom, mut right, mut top, mut depth] = match face {
            Direction::Down => [x1, z1, x2, z2, y1],
            Direction::Up => [x1, 1. - z2, x2, 1. - z1, 1. - y2],
            Direction::North => [1. - x2, y1, 1. - x1, y2, z1],
            Direction::South => [x1, y1, x2, y2, 1. - z2],
            Direction::West => [z1, y1, z2, y2, x1],
            Direction::East => [1. - z2, y1, 1. - z1, y2, 1. - x2],
        };
        let cull = if depth.abs() < 0.00001 {
            depth = 0.;
            Some(face)
        } else {
            None
        };
        let positions = match face {
            Direction::Up | Direction::Down => {
                if face == Direction::Up {
                    depth = 1. - depth;
                    top = 1. - top;
                    bottom = 1. - bottom;
                }
                [
                    [left, depth, top],
                    [left, depth, bottom],
                    [right, depth, bottom],
                    [right, depth, top],
                ]
            }
            Direction::East | Direction::West => {
                if face == Direction::East {
                    depth = 1. - depth;
                    left = 1. - left;
                    right = 1. - right;
                }
                [
                    [depth, top, left],
                    [depth, bottom, left],
                    [depth, bottom, right],
                    [depth, top, right],
                ]
            }
            Direction::South | Direction::North => {
                if face == Direction::South {
                    depth = 1. - depth;
                    left = 1. - left;
                    right = 1. - right;
                }
                [
                    [1. - left, top, depth],
                    [1. - left, bottom, depth],
                    [1. - right, bottom, depth],
                    [1. - right, top, depth],
                ]
            }
        };
        self.output.push(Quad {
            texture: texture.id.clone(),
            face,
            cull,
            positions,
            uv,
            color: self.color,
            emissive: self.emissive,
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_cube_has_source_winding_cull_faces_rotations_and_fullbright_color() {
        let mut b = CubeBuilder::default();
        b.set_texture(Texture::unit("test"));
        b.set_rotation(Direction::Up, 1).unwrap();
        b.set_rgb(0x123456);
        b.emissive = true;
        b.add_cube([0., 0., 0., 16., 16., 16.]);
        assert_eq!(b.output.len(), 6);
        assert!(
            b.output
                .iter()
                .all(|q| q.cull == Some(q.face) && q.color == 0xff123456 && q.emissive)
        );
        assert_eq!(
            b.output[0].positions,
            [[0., 0., 1.], [0., 0., 0.], [1., 0., 0.], [1., 0., 1.]]
        );
        assert_eq!(b.output[1].uv, [[0., 1.], [1., 1.], [1., 0.], [0., 0.]]);
        assert!(b.set_rotation(Direction::Up, 4).is_err());
    }
}
