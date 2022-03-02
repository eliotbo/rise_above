use bevy::{
    core::FloatOrd,
    core_pipeline::Transparent2d,
    ecs::system::lifetimeless::{Read, SQuery, SRes},
    ecs::system::SystemParamItem,
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::GpuBufferInfo,
        render_asset::RenderAssets,
        render_component::{ComponentUniforms, DynamicUniformIndex, UniformComponentPlugin},
        render_component::{ExtractComponent, ExtractComponentPlugin},
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{std140::AsStd140, *},
        renderer::RenderDevice,
        view::VisibleEntities,
        view::{ComputedVisibility, Msaa},
        RenderApp, RenderStage,
    },
    sprite::{
        Mesh2dHandle, Mesh2dPipeline, Mesh2dPipelineKey, Mesh2dUniform, SetMesh2dBindGroup,
        SetMesh2dViewBindGroup,
    },
};

pub mod inputs;
pub use inputs::*;
pub mod selection;
pub use selection::*;

pub mod encoding;
pub use encoding::*;

use bevy_inspector_egui::*;
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

pub struct SpawnAllEvent;

pub const ATTR_SIZE: usize = 13;

#[derive(Component)]
pub struct SelectedText;

#[derive(Component, Default)]
pub struct SelectedBall {
    pub instance: usize,
    pub ball_index: Option<usize>,
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
impl ExtractComponent for MarkerInstanceMatData {
    type Query = &'static MarkerInstanceMatData;
    type Filter = ();

    fn extract_component(item: bevy::ecs::query::QueryItem<Self::Query>) -> Self {
        MarkerInstanceMatData(item.0.clone())
    }
}

#[derive(Component, Default)]
pub(crate) struct MarkerMesh2d;

#[derive(Component, Clone, AsStd140)]
pub struct CharacterUniform {
    pub character_size: f32,
    /// When the ```marker_point_color``` field is different from the ```color``` field,
    /// there is a small visible circle within the marker. ```core_size``` controls the size of the circle.
    pub core_size: f32,
    pub zoom: f32,
    pub time: f32,
    /// Size of the instanced square quad for one marker.
    pub quad_size: f32,

    /// Shows a black contour around the marker if the value is > 0.5.
    pub contour: f32,
    pub inner_canvas_size_in_pixels: Vec2,
    pub canvas_position: Vec2,
    pub color: Vec4,

    /// Color of the small circle within the marker.
    pub character_point_color: Vec4,
}

// TODO: we have instance data, but we don't use it at the moment.
// One use case would be to have marker size as an additional dimension.

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

#[test]
pub fn encode_and_decode() {
    let d = InstanceDataNotEncoded::default();

    let mut m = MarkerInstanceData {
        group_position: [0, 1, 2],
        scale: 1,
        color: [0, 1, 2, 3],
        data: [[0, 1]; ATTR_SIZE * 2],
    };

    m.set_all_data(vec![d]);

    // println!(" m.get_max_size(0)  : {:?}", m.get_max_size(0));
    // println!(" m.get_frequency(0) : {:?}", m.get_frequency(0));
    // println!(" m.get_noise(0)     : {:?}", m.get_noise(0));
    // println!(" m.get_min_size(0)  : {:?}", m.get_min_size(0));
    // println!(" m.get_morph(0)     : {:?}", m.get_morph(0));
    // println!(" m.get_core_size(0) : {:?}", m.get_core_size(0));

    // assert!(m.get_pos(0) - Vec2::new(0.398, 0.456));
    assert!(m.get_max_size(0) - 3.0 < 0.0001);
    assert!(m.get_frequency(0) - 4.0 < 0.0001);
    assert!(m.get_noise(0) - 5.0 < 0.0001);
    assert!(m.get_min_size(0) - 6.0 < 0.0001);
    assert!(m.get_morph(0) - 7.0 < 0.0001);
    assert!(m.get_core_size(0) - 1.0 < 0.0001);
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CharacterSaveFormat {
    pub group_position: [u32; 3],
    pub scale: u32,
    pub color: [u32; 4],
    pub data: Vec<InstanceDataNotEncoded>,
}

// impl From<MarkerInstanceMatData> for CharacterSaveFormat {
//     fn from(m: MarkerInstanceMatData) -> Self {
//         CharacterSaveFormat {
//             data: m
//                 .0
//                 .into_iter()
//                 .enumerate()
//                 .map(|(k, d)| d.get_all(k))
//                 .collect(),
//         }
//     }
// }

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

// use bevy_inspector_egui::{egui, Inspectable, InspectorPlugin};
/// Instance data format before the data is encoded for sending to the GPU
#[derive(Inspectable, Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct InstanceDataNotEncoded {
    // #[inspectable(speed = 0.003, min = Vec2::ZERO, max = Vec2::ONE)]
    #[inspectable(speed = 0.003, min = Vec2::splat(-0.5), max = Vec2::splat(0.5))]
    pub pos: Vec2,

    // pub max_size: u8,
    // pub frequency: u8,
    // pub noise: u8,

    // pub min_size: u8,
    // pub morph: u8,
    // pub core_size: u8,
    #[inspectable(speed = 0.003, min = 0.0, max = 1.0)]
    pub max_size: f32,
    #[inspectable(speed = 0.003, min = 0.0, max = 1.0)]
    pub frequency: f32,
    #[inspectable(speed = 0.003, min = 0.0, max = 0.9)]
    pub noise: f32,

    #[inspectable(speed = 0.003, min = 0.0, max = 1.0)]
    pub min_size: f32,
    #[inspectable(speed = 0.003, min = 0.0, max = 1.0)]
    pub morph: f32,
    #[inspectable(speed = 0.003, min = 0.0, max = 1.0)]
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

/// Custom pipeline for 2d meshes with vertex colors
pub(crate) struct MarkerMesh2dPipeline {
    /// this pipeline wraps the standard [`Mesh2dPipeline`]
    mesh2d_pipeline: Mesh2dPipeline,
    pub custom_uniform_layout: BindGroupLayout,
    // pub shader: Handle<Shader>,
    // material_layout: BindGroupLayout,
}

impl FromWorld for MarkerMesh2dPipeline {
    fn from_world(world: &mut World) -> Self {
        let mesh2d_pipeline = Mesh2dPipeline::from_world(world).clone();

        let render_device = world.get_resource::<RenderDevice>().unwrap();

        let custom_uniform_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: BufferSize::new(
                            CharacterUniform::std140_size_static() as u64
                        ),
                    },
                    count: None,
                }],
                label: Some("markers_uniform_layout"),
            });

        // let world = world.cell();
        // let asset_server = world.get_resource::<AssetServer>().unwrap();

        // let shader = asset_server.load("../assets/shaders/markers.wgsl");

        // let _result = asset_server.watch_for_changes();

        Self {
            mesh2d_pipeline,
            custom_uniform_layout,
            // shader,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct MarkerPipelineKey {
    mesh: Mesh2dPipelineKey,
    shader_handle: Handle<Shader>,
}

impl SpecializedPipeline for MarkerMesh2dPipeline {
    type Key = MarkerPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut descriptor = self.mesh2d_pipeline.specialize(key.mesh);

        descriptor.vertex.shader = key.shader_handle.clone();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: std::mem::size_of::<MarkerInstanceData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: (0..(0 + ATTR_SIZE.min(29)))
                .map(|i| VertexAttribute {
                    format: VertexFormat::Uint32x4,
                    offset: i as u64 * 16,
                    shader_location: i as u32 + 3,
                })
                .collect(),
        });
        descriptor.fragment.as_mut().unwrap().shader = key.shader_handle.clone();
        descriptor.layout = Some(vec![
            self.mesh2d_pipeline.view_layout.clone(),
            self.mesh2d_pipeline.mesh_layout.clone(),
            self.custom_uniform_layout.clone(),
        ]);

        descriptor
    }
}

// This specifies how to render a colored 2d mesh
type DrawMarkerMesh2d = (
    // Set the pipeline
    SetItemPipeline,
    // Set the view uniform as bind group 0
    SetMesh2dViewBindGroup<0>,
    // Set the mesh uniform as bind group 1
    SetMesh2dBindGroup<1>,
    // Set the marker uniform as bind group 2
    SetCharacterUniformBindGroup<2>,
    // Draw the mesh
    DrawMarkerMeshInstanced,
);

pub struct MarkerMesh2dPlugin;

pub(crate) struct MarkerShaderHandle(pub Handle<Shader>);

pub const MARKER_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 9826351934109932579);

impl Plugin for MarkerMesh2dPlugin {
    fn build(&self, app: &mut App) {
        // let asset_server = app.world.get_resource_mut::<AssetServer>().unwrap();

        // let shader_typed_handle = asset_server.load("character.wgsl");
        // let _ = asset_server.watch_for_changes().unwrap();

        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();

        let handle_untyped = MARKER_SHADER_HANDLE.clone();

        shaders.set_untracked(
            handle_untyped.clone(),
            Shader::from_wgsl(include_str!("character.wgsl")),
        );

        let shader_typed_handle = shaders.get_handle(handle_untyped);

        app.add_plugin(UniformComponentPlugin::<CharacterUniform>::default());
        app.add_plugin(ExtractComponentPlugin::<MarkerInstanceMatData>::default());

        // Register our custom draw function and pipeline, and add our render systems
        let render_app = app.get_sub_app_mut(RenderApp).unwrap();
        render_app
            .add_render_command::<Transparent2d, DrawMarkerMesh2d>()
            .init_resource::<MarkerMesh2dPipeline>()
            .init_resource::<SpecializedPipelines<MarkerMesh2dPipeline>>()
            .insert_resource(MarkerShaderHandle(shader_typed_handle))
            // .insert_resource(MarkerShaderHandle(shader_handle))
            .add_system_to_stage(RenderStage::Prepare, prepare_instance_buffers)
            .add_system_to_stage(RenderStage::Extract, extract_colored_mesh2d)
            .add_system_to_stage(RenderStage::Queue, queue_marker_uniform_bind_group)
            .add_system_to_stage(RenderStage::Queue, queue_colored_mesh2d);
    }
}

/// Extract CharacterUniform
fn extract_colored_mesh2d(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    query: Query<(Entity, &CharacterUniform, &ComputedVisibility), With<MarkerInstanceMatData>>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, custom_uni, computed_visibility) in query.iter() {
        if !computed_visibility.is_visible {
            continue;
        }
        values.push((entity, (custom_uni.clone(), MarkerMesh2d)));
    }
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}

fn prepare_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &MarkerInstanceMatData)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, instance_data) in query.iter() {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("marker instance data buffer"),
            contents: bytemuck::cast_slice(instance_data.0.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        commands.entity(entity).insert(MarkerInstanceBuffer {
            buffer,
            length: instance_data.0.len(),
        });
    }
}

struct CharacterUniformBindGroup {
    pub value: BindGroup,
}

fn queue_marker_uniform_bind_group(
    mut commands: Commands,
    mesh2d_pipeline: Res<MarkerMesh2dPipeline>,
    render_device: Res<RenderDevice>,
    mesh2d_uniforms: Res<ComponentUniforms<CharacterUniform>>,
) {
    if let Some(binding) = mesh2d_uniforms.uniforms().binding() {
        commands.insert_resource(CharacterUniformBindGroup {
            value: render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: binding,
                }],
                label: Some("MarkersUniform_bind_group"),
                layout: &mesh2d_pipeline.custom_uniform_layout,
            }),
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn queue_colored_mesh2d(
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    colored_mesh2d_pipeline: Res<MarkerMesh2dPipeline>,
    mut pipelines: ResMut<SpecializedPipelines<MarkerMesh2dPipeline>>,
    mut pipeline_cache: ResMut<RenderPipelineCache>,
    msaa: Res<Msaa>,
    render_meshes: Res<RenderAssets<Mesh>>,
    shader_handle: Res<MarkerShaderHandle>,
    colored_mesh2d: Query<(&Mesh2dHandle, &Mesh2dUniform), With<MarkerInstanceMatData>>,
    mut views: Query<(&VisibleEntities, &mut RenderPhase<Transparent2d>)>,
) {
    if colored_mesh2d.is_empty() {
        return;
    }

    // Iterate each view (a camera is a view)
    for (visible_entities, mut transparent_phase) in views.iter_mut() {
        let draw_colored_mesh2d = transparent_draw_functions
            .read()
            .get_id::<DrawMarkerMesh2d>()
            .unwrap();

        // let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples);

        let mesh_key = MarkerPipelineKey {
            mesh: Mesh2dPipelineKey::from_msaa_samples(msaa.samples),
            shader_handle: shader_handle.0.clone(),
        };

        // Queue all entities visible to that view
        for visible_entity in &visible_entities.entities {
            if let Ok((mesh2d_handle, mesh2d_uniform)) = colored_mesh2d.get(*visible_entity) {
                let mut mesh2d_key = mesh_key.clone();
                if let Some(mesh) = render_meshes.get(&mesh2d_handle.0) {
                    mesh2d_key.mesh |=
                        Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology);
                }

                let pipeline_id =
                    pipelines.specialize(&mut pipeline_cache, &colored_mesh2d_pipeline, mesh2d_key);

                let mesh_z = mesh2d_uniform.transform.w_axis.z;
                transparent_phase.add(Transparent2d {
                    entity: *visible_entity,
                    draw_function: draw_colored_mesh2d,
                    pipeline: pipeline_id,
                    sort_key: FloatOrd(mesh_z),
                    batch_range: None,
                });
            }
        }
    }
}

struct SetCharacterUniformBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetCharacterUniformBindGroup<I> {
    type Param = (
        SRes<CharacterUniformBindGroup>,
        SQuery<Read<DynamicUniformIndex<CharacterUniform>>>,
    );
    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        (mesh2d_bind_group, mesh2d_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mesh2d_index = mesh2d_query.get(item).unwrap();

        pass.set_bind_group(
            I,
            &mesh2d_bind_group.into_inner().value,
            &[mesh2d_index.index()],
        );
        RenderCommandResult::Success
    }
}

#[derive(Component)]
struct MarkerInstanceBuffer {
    buffer: Buffer,
    length: usize,
}

struct DrawMarkerMeshInstanced;
impl EntityRenderCommand for DrawMarkerMeshInstanced {
    type Param = (
        SRes<RenderAssets<Mesh>>,
        SQuery<Read<Mesh2dHandle>>,
        SQuery<Read<MarkerInstanceBuffer>>,
    );
    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        (meshes, mesh2d_query, instance_buffer_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mesh_handle = &mesh2d_query.get(item).unwrap().0;
        let instance_buffer = instance_buffer_query.get(item).unwrap();

        let gpu_mesh = match meshes.into_inner().get(mesh_handle) {
            Some(gpu_mesh) => gpu_mesh,
            None => return RenderCommandResult::Failure,
        };

        pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));

        pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
        match &gpu_mesh.buffer_info {
            GpuBufferInfo::Indexed {
                buffer,
                index_format,
                count,
            } => {
                pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                pass.draw_indexed(0..*count, 0, 0..instance_buffer.length as u32);
            }
            GpuBufferInfo::NonIndexed { vertex_count } => {
                pass.draw_indexed(0..*vertex_count, 0, 0..instance_buffer.length as u32);
            }
        }
        RenderCommandResult::Success
    }
}
