// // pub mod canvas;
// pub mod canvas_actions;
// #[allow(unused_imports)]
// pub use canvas_actions::*;

use bevy::{
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::{PrepareAssetError, RenderAsset},
        render_resource::{
            std140::{AsStd140, Std140},
            *,
        },
        renderer::RenderDevice,
    },
    sprite::Material2d,
    sprite::Material2dPipeline,
};

/// Canvas shader parameters
#[derive(TypeUuid, Debug, Clone, Component, AsStd140)]
#[uuid = "1e08866c-0b8a-437e-8bae-28844b21127e"]
#[allow(non_snake_case)]
pub struct SelectionMaterial {
    pub pos: Vec2,
    pub color: Vec4,
    pub radius: f32,
    // /// Mouse position in the reference frame of the graph, corresponding to its axes coordinates
    // pub mouse_pos: Vec2,
    // pub tick_period: Vec2,

    // /// Extreme points of the canvas
    // pub bounds: PlotCanvasBounds,

    // pub time: f32,
    // pub zoom: f32,
    // pub size: Vec2,
    // pub outer_border: Vec2,
    // pub position: Vec2,
    // pub show_target: f32,
    // pub hide_contour: f32,
    // pub target_pos: Vec2,

    // pub background_color1: Vec4,
    // pub background_color2: Vec4,
    // pub target_color: Vec4,

    // pub show_grid: f32,
    // pub show_axes: f32,
}

impl Default for SelectionMaterial {
    fn default() -> Self {
        SelectionMaterial {
            pos: Vec2::ZERO,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            radius: 5.0,
            // mouse_pos: Vec2::ZERO,
            // tick_period: plot.tick_period,
            // bounds: plot.bounds.clone(),
            // time: 0.0,
            // zoom: 1.0,
            // size: plot.canvas_size,
            // outer_border: plot.outer_border,
            // position: plot.canvas_position,
            // show_target: if plot.show_target && plot.target_toggle {
            //     1.0
            // } else {
            //     0.0
            // },
            // hide_contour: if plot.hide_contour { 1.0 } else { 0.0 },
            // target_pos: Vec2::ZERO,
            // background_color1: col_to_vec4(plot.background_color1),
            // background_color2: col_to_vec4(plot.background_color2),
            // target_color: col_to_vec4(plot.target_color),
            // show_grid: if plot.show_grid { 1.0 } else { 0.0 },
            // show_axes: if plot.show_axes { 1.0 } else { 0.0 },
        }
    }
}

#[derive(Clone)]
pub struct GpuSelectionMaterial {
    _buffer: Buffer,
    bind_group: BindGroup,
}

pub struct SelectionMesh2dPlugin;

pub const CANVAS_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 11248119131754755027);

impl Plugin for SelectionMesh2dPlugin {
    fn build(&self, app: &mut App) {
        // // let mut shaders = world.get_resource_mut::<Assets<Shader>>().unwrap();
        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();

        let handle_untyped = CANVAS_SHADER_HANDLE.clone();

        shaders.set_untracked(
            handle_untyped.clone(),
            Shader::from_wgsl(include_str!("selection.wgsl")),
        );

        // let mut asset_server = app.world.get_resource_mut::<AssetServer>().unwrap();

        // let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();

        // let _shader_typed_handle: Handle<Shader> = asset_server.load("assets/selection.wgsl");
        // let _ = asset_server.watch_for_changes().unwrap();

        // //

        // // at the moment, there seems to be no way to include a font in the crate
        // let mut fonts = app.world.get_resource_mut::<Assets<Font>>().unwrap();
    }
}

impl Material2d for SelectionMaterial {
    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        let handle_untyped = CANVAS_SHADER_HANDLE.clone();
        let shader_handle: Handle<Shader> = handle_untyped.typed::<Shader>();

        // println!("LOADED LOADED LOADED LOADED LOADED LOADED LOADED LOADED LOADED LOADED LOADED");

        // let shader_handle: Handle<Shader> = asset_server.load("selection.wgsl");
        // let _ = asset_server.watch_for_changes().unwrap();

        Some(shader_handle)
    }

    fn bind_group(render_asset: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &render_asset.bind_group
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(
                        SelectionMaterial::std140_size_static() as u64
                    ),
                },
                count: None,
            }],
            label: None,
        })
    }
}

impl RenderAsset for SelectionMaterial {
    type ExtractedAsset = SelectionMaterial;
    type PreparedAsset = GpuSelectionMaterial;
    type Param = (SRes<RenderDevice>, SRes<Material2dPipeline<Self>>);
    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (render_device, material_pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let custom_material_std140 = extracted_asset.as_std140();
        let custom_material_bytes = custom_material_std140.as_bytes();

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: custom_material_bytes,
            label: None,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
            layout: &material_pipeline.material2d_layout,
        });

        Ok(GpuSelectionMaterial {
            _buffer: buffer,
            bind_group,
        })
    }
}
