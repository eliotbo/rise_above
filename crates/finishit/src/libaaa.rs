use bevy::prelude::*;

// use bevy_inspector_egui::*;
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

// mod encoding;
use crate::encoding::*;

pub struct SpawnAllEvent;

pub const ATTR_SIZE: usize = 13;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CharacterSaveFormat {
    pub group_position: [u32; 3],
    pub scale: u32,
    pub color: [u32; 4],
    pub data: Vec<InstanceDataNotEncoded>,
}

// use bevy_inspector_egui::{egui, Inspectable, InspectorPlugin};
/// Instance data format before the data is encoded for sending to the GPU
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct InstanceDataNotEncoded {
    pub pos: Vec2,

    pub max_size: f32,

    pub frequency: f32,

    pub noise: f32,

    pub min_size: f32,

    pub morph: f32,

    pub core_size: f32,

    pub is_joint: bool,
}

impl Default for InstanceDataNotEncoded {
    fn default() -> Self {
        InstanceDataNotEncoded {
            pos: Vec2::splat(10000000.0),
            max_size: 0.8,
            frequency: 0.6,
            noise: 0.05,

            min_size: 0.7,
            morph: 0.5,
            core_size: 0.7,

            is_joint: false,
            // max_size: 75,
            // frequency: 33,
            // noise: 15,

            // min_size: 65,
            // morph: 55,
            // core_size: 25,
        }
    }
}

impl InstanceDataNotEncoded {
    pub fn new_joint_at_pos(pos: Vec2) -> Self {
        InstanceDataNotEncoded {
            pos,
            max_size: 0.4,
            frequency: 0.4,
            noise: 1.00,

            min_size: 0.3,
            morph: 0.2,
            core_size: 0.0,

            is_joint: true,
            // max_size: 75,
            // frequency: 33,
            // noise: 15,

            // min_size: 65,
            // morph: 55,
            // core_size: 25,
        }
    }
}

#[derive(Component, Clone)]
pub struct MarkerInstanceMatData(pub Vec<MarkerInstanceData>);

impl MarkerInstanceMatData {
    pub fn within_rect(&self, cursor_pos: Vec2, quad_size: f32) -> Option<usize> {
        //
        // self.0.iter().enumerate().for_each(|(i, marker)| {
        for (i, marker) in self.0.iter().enumerate() {
            let pos = marker.get_pos(i);
            if cursor_pos.x < pos.x + quad_size / 2.0
                && cursor_pos.x > pos.x - quad_size / 2.0
                && cursor_pos.y < pos.y + quad_size / 2.0
                && cursor_pos.y > pos.y - quad_size / 2.0
            {
                //
                return Some(i);
            }
            // });
        }

        None
    }

    pub fn within_rect_delta(&self, cursor_pos: Vec2, quad_size: f32) -> bool {
        //
        // self.0.iter().enumerate().for_each(|(i, marker)| {
        for (i, marker) in self.0.iter().enumerate() {
            let pos = marker.get_pos(i);
            if cursor_pos.x < pos.x + quad_size / 2.0
                && cursor_pos.x > pos.x - quad_size / 2.0
                && cursor_pos.y < pos.y + quad_size / 2.0
                && cursor_pos.y > pos.y - quad_size / 2.0
            {
                //
                return true;
            }
            // });
        }

        return false;
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct MarkerInstanceData {
    pub group_position: [u32; 3],
    pub scale: u32,
    pub color: [u32; 4],
    pub data: [[u32; 2]; ATTR_SIZE * 2],
    // pub data: [f32; ATTR_SIZE * 4],
    // col2: [f32; 4],
}

impl MarkerInstanceData {
    pub fn set_group_position(&mut self, pos: Vec3) {
        pos.x
            .encode_to_u32_with_precision(&mut self.group_position[0], 32, 32);
        pos.y
            .encode_to_u32_with_precision(&mut self.group_position[1], 32, 32);
        pos.z
            .encode_to_u32_with_precision(&mut self.group_position[2], 32, 32);
    }

    pub fn set_scale(&mut self, scale: f32) {
        scale.encode_to_u32_with_precision(&mut self.scale, 32, 32);
    }

    pub fn set_color(&mut self, color: Vec4) {
        color
            .x
            .encode_to_u32_with_precision(&mut self.color[0], 32, 32);
        color
            .y
            .encode_to_u32_with_precision(&mut self.color[1], 32, 32);
        color
            .z
            .encode_to_u32_with_precision(&mut self.color[2], 32, 32);
        color
            .w
            .encode_to_u32_with_precision(&mut self.color[3], 32, 32);
    }

    pub fn delete(&mut self, index: usize) {
        self.set_pos(Vec2::splat(1000000000.0), index);
    }

    pub fn set_data(&mut self, index: usize, v: InstanceDataNotEncoded) {
        // self.data[i * 2] = [v.pos.x as u32, v.pos.y as u32];

        let mut encoded_x = 0_u32;
        //
        (v.pos.x + 0.5).encode_to_u32_with_precision(&mut encoded_x, 32, 16);
        v.max_size
            .encode_to_u32_with_precision(&mut encoded_x, 16, 8);
        v.frequency
            .encode_to_u32_with_precision(&mut encoded_x, 8, 4);

        // If the node is a joint, encode the noise to its maximum value.
        // The shader decodes the noise and check if it's > 0.99
        if v.is_joint {
            1.0.encode_to_u32_with_precision(&mut encoded_x, 4, 4);
        } else {
            v.noise.encode_to_u32_with_precision(&mut encoded_x, 4, 4);
        }

        let mut encoded_y = 0_u32;
        (v.pos.y + 0.5).encode_to_u32_with_precision(&mut encoded_y, 32, 16);
        v.min_size
            .encode_to_u32_with_precision(&mut encoded_y, 16, 8);
        v.morph.encode_to_u32_with_precision(&mut encoded_y, 8, 4);
        v.core_size
            .encode_to_u32_with_precision(&mut encoded_y, 4, 4);

        self.data[index][0] = encoded_x;
        self.data[index][1] = encoded_y;
    }

    pub fn new(vs: Vec<InstanceDataNotEncoded>) -> Self {
        let mut instances = MarkerInstanceData {
            group_position: [0; 3],
            scale: u32::MAX,
            color: [u32::MAX / 2; 4],
            data: [[0; 2]; ATTR_SIZE * 2],
        };

        instances.set_all_data(vs);
        instances
    }

    pub fn set_all_data(&mut self, vs: Vec<InstanceDataNotEncoded>) {
        for (i, v) in vs.iter().enumerate() {
            // self.data[i * 2] = [v.pos.x as u32, v.pos.y as u32];

            let mut encoded_x = 0_u32;
            //
            (v.pos.x + 0.5).encode_to_u32_with_precision(&mut encoded_x, 32, 16);
            v.max_size
                .encode_to_u32_with_precision(&mut encoded_x, 16, 8);
            v.frequency
                .encode_to_u32_with_precision(&mut encoded_x, 8, 4);
            if v.is_joint {
                1.0.encode_to_u32_with_precision(&mut encoded_x, 4, 4);
            } else {
                v.noise.encode_to_u32_with_precision(&mut encoded_x, 4, 4);
            }

            let mut encoded_y = 0_u32;
            (v.pos.y + 0.5).encode_to_u32_with_precision(&mut encoded_y, 32, 16);
            v.min_size
                .encode_to_u32_with_precision(&mut encoded_y, 16, 8);
            v.morph.encode_to_u32_with_precision(&mut encoded_y, 8, 4);
            v.core_size
                .encode_to_u32_with_precision(&mut encoded_y, 4, 4);

            self.data[i][0] = encoded_x;
            self.data[i][1] = encoded_y;
        }
    }

    pub fn set_pos(&mut self, pos: Vec2, index: usize) {
        (pos.x + 0.5).encode_to_u32_with_precision(&mut self.data[index][0], 32, 16);
        (pos.y + 0.5).encode_to_u32_with_precision(&mut self.data[index][1], 32, 16);
    }

    pub fn set_max_size(&mut self, max_size: f32, index: usize) {
        max_size.encode_to_u32_with_precision(&mut self.data[index][0], 16, 8);
    }

    pub fn set_frequency(&mut self, frequency: f32, index: usize) {
        frequency.encode_to_u32_with_precision(&mut self.data[index][0], 8, 4);
    }

    pub fn set_noise(&mut self, noise: f32, index: usize) {
        noise.encode_to_u32_with_precision(&mut self.data[index][0], 4, 4);
    }

    pub fn set_min_size(&mut self, min_size: f32, index: usize) {
        min_size.encode_to_u32_with_precision(&mut self.data[index][1], 16, 8);
    }

    pub fn set_morph(&mut self, morph: f32, index: usize) {
        morph.encode_to_u32_with_precision(&mut self.data[index][1], 8, 4);
    }

    pub fn set_core_size(&mut self, core_size: f32, index: usize) {
        core_size.encode_to_u32_with_precision(&mut self.data[index][1], 4, 4);
    }

    pub fn set_is_joint(&mut self, is_joint: bool, index: usize) {
        if is_joint {
            1.0.encode_to_u32_with_precision(&mut self.data[index][1], 4, 4);
        } else {
            0.0.encode_to_u32_with_precision(&mut self.data[index][1], 4, 4);
        }
    }

    pub fn get_pos(&self, index: usize) -> Vec2 {
        let mut v = Vec2::ZERO;
        v.x = self.data[index][0].decode(32, 16) - 0.5;
        v.y = self.data[index][1].decode(32, 16) - 0.5;
        v
    }

    pub fn get_max_size(&self, index: usize) -> f32 {
        self.data[index][0].decode(16, 8) as f32
    }

    pub fn get_frequency(&self, index: usize) -> f32 {
        self.data[index][0].decode(8, 4) as f32
    }

    pub fn get_noise(&self, index: usize) -> f32 {
        self.data[index][0].decode(4, 4) as f32
    }

    pub fn get_min_size(&self, index: usize) -> f32 {
        self.data[index][1].decode(16, 8) as f32
    }

    pub fn get_morph(&self, index: usize) -> f32 {
        self.data[index][1].decode(8, 4) as f32
    }

    pub fn get_core_size(&self, index: usize) -> f32 {
        self.data[index][1].decode(4, 4) as f32
    }

    pub fn get_all(&self, index: usize) -> InstanceDataNotEncoded {
        let noise = self.get_noise(index);
        InstanceDataNotEncoded {
            pos: self.get_pos(index),
            // max_size: self.get_max_size(index).to_u8(),
            // frequency: self.get_frequency(index).to_u8(),
            // noise: self.get_noise(index).to_u8(),
            // min_size: self.get_min_size(index).to_u8(),
            // morph: self.get_morph(index).to_u8(),
            // core_size: self.get_core_size(index).to_u8(),
            max_size: self.get_max_size(index),
            frequency: self.get_frequency(index),
            noise,
            min_size: self.get_min_size(index),
            morph: self.get_morph(index),
            core_size: self.get_core_size(index),
            is_joint: if noise > 0.99 { true } else { false },
        }
    }

    pub fn get_group_position(&self) -> Vec3 {
        Vec3::new(
            self.group_position[0].decode(32, 32) as f32 - 0.5,
            self.group_position[1].decode(32, 32) as f32 - 0.5,
            self.group_position[2].decode(32, 32) as f32 - 0.5,
        )
    }

    // TODO: scale should map the range [0, 1] to [0, 0.5] and [1, 100] to [0.5, 1]
    pub fn get_scale(&self) -> f32 {
        self.scale.decode(32, 32) as f32
    }

    pub fn get_color(&self) -> Vec4 {
        Vec4::new(
            self.color[0].decode(32, 32) as f32,
            self.color[1].decode(32, 32) as f32,
            self.color[2].decode(32, 32) as f32,
            self.color[3].decode(32, 32) as f32,
        )
    }
}

impl Into<CharacterSaveFormat> for MarkerInstanceMatData {
    fn into(self) -> CharacterSaveFormat {
        // choses the 0th entity
        CharacterSaveFormat {
            group_position: self.0[0].group_position,
            scale: self.0[0].scale,
            color: self.0[0].color,
            data: (0..ATTR_SIZE * 2).map(|k| self.0[0].get_all(k)).collect(),
        }
    }
}

impl Into<MarkerInstanceMatData> for CharacterSaveFormat {
    fn into(self) -> MarkerInstanceMatData {
        MarkerInstanceMatData(vec![MarkerInstanceData {
            group_position: self.group_position.into(),
            scale: self.scale.into(),
            color: self.color.into(),
            data: MarkerInstanceData::new(self.data).data,
        }])
    }
}
