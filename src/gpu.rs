//! Optional WGPU resources for Illustrator-parity effects.
//!
//! The CPU scene renderer preserves the full appearance stack, but true Illustrator parity for
//! blend modes, blur chains, masks, and isolated groups requires offscreen GPU passes. This module
//! exposes the initialization hook and shader resources used by those passes while keeping the crate
//! usable without WGPU by default.

use std::collections::HashMap;

use egui::PaintCallbackInfo;
use egui_wgpu::{wgpu, CallbackResources, CallbackTrait, ScreenDescriptor};

/// GPU resources installed into `egui_wgpu::Renderer::callback_resources`.
pub struct GpuEffectsResources {
    pub blend_pipeline: wgpu::RenderPipeline,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub uniform_bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    transparent_base_bind_group: wgpu::BindGroup,
    uploaded_composites: HashMap<u64, UploadedCompositeTexture>,
    frame_counter: u64,
}

struct UploadedCompositeTexture {
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    size: [u32; 2],
    last_used_frame: u64,
}

/// Per-frame GPU callback that paints a pre-composited RGBA texture through the
/// same wgpu callback pipeline used by richer blend/mask passes.
pub struct GpuCompositeCallback {
    id: u64,
    size: [u32; 2],
    rgba: Vec<u8>,
}

impl GpuCompositeCallback {
    pub fn new(id: u64, size: [u32; 2], rgba: Vec<u8>) -> Self {
        Self { id, size, rgba }
    }
}

/// Initialize GPU effect resources from an egui-wgpu render state.
///
/// Call this once from an eframe app's creation context when WGPU rendering is enabled:
///
/// ```ignore
/// if let Some(render_state) = cc.wgpu_render_state.as_ref() {
///     egui_expressive::init_gpu_effects(render_state);
/// }
/// ```
pub fn init_gpu_effects(render_state: &egui_wgpu::RenderState) {
    let resources = create_gpu_effects_resources(&render_state.device, render_state.target_format);
    render_state
        .renderer
        .write()
        .callback_resources
        .insert(resources);
}

fn create_gpu_effects_resources(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
) -> GpuEffectsResources {
    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("egui_expressive_blend_texture_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("egui_expressive_blend_sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });

    let transparent_base_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("egui_expressive_transparent_base_texture"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let transparent_base_view = transparent_base_texture.create_view(&Default::default());
    let transparent_base_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("egui_expressive_transparent_base_bg"),
        layout: &texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&transparent_base_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    let uniform_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("egui_expressive_blend_uniform_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("egui_expressive_blend_shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("draw/blend_shader.wgsl").into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("egui_expressive_blend_pipeline_layout"),
        bind_group_layouts: &[
            Some(&texture_bind_group_layout),
            Some(&texture_bind_group_layout),
            Some(&uniform_bind_group_layout),
        ],
        immediate_size: 0,
    });

    let blend_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("egui_expressive_blend_pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    GpuEffectsResources {
        blend_pipeline,
        texture_bind_group_layout,
        uniform_bind_group_layout,
        sampler,
        transparent_base_bind_group,
        uploaded_composites: HashMap::new(),
        frame_counter: 0,
    }
}

impl CallbackTrait for GpuCompositeCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let Some(resources) = callback_resources.get_mut::<GpuEffectsResources>() else {
            return Vec::new();
        };
        if self.size[0] == 0 || self.size[1] == 0 || self.rgba.is_empty() {
            return Vec::new();
        }

        resources.frame_counter += 1;
        let current_frame = resources.frame_counter;

        if resources.uploaded_composites.len() > 64 {
            let mut oldest_id = None;
            let mut oldest_frame = u64::MAX;
            for (id, tex) in &resources.uploaded_composites {
                if tex.last_used_frame < oldest_frame {
                    oldest_frame = tex.last_used_frame;
                    oldest_id = Some(*id);
                }
            }
            if let Some(id) = oldest_id {
                resources.uploaded_composites.remove(&id);
            }
        }

        let recreate = resources
            .uploaded_composites
            .get(&self.id)
            .map(|uploaded| uploaded.size != self.size)
            .unwrap_or(true);

        if recreate {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("egui_expressive_composite_texture"),
                size: wgpu::Extent3d {
                    width: self.size[0],
                    height: self.size[1],
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let view = texture.create_view(&Default::default());
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("egui_expressive_composite_texture_bg"),
                layout: &resources.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&resources.sampler),
                    },
                ],
            });
            let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("egui_expressive_composite_uniforms"),
                size: 16,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("egui_expressive_composite_uniform_bg"),
                layout: &resources.uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }],
            });
            resources.uploaded_composites.insert(
                self.id,
                UploadedCompositeTexture {
                    texture,
                    bind_group,
                    uniform_buffer,
                    uniform_bind_group,
                    size: self.size,
                    last_used_frame: current_frame,
                },
            );
        }

        if let Some(uploaded) = resources.uploaded_composites.get_mut(&self.id) {
            uploaded.last_used_frame = current_frame;
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &uploaded.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &self.rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * self.size[0]),
                    rows_per_image: Some(self.size[1]),
                },
                wgpu::Extent3d {
                    width: self.size[0],
                    height: self.size[1],
                    depth_or_array_layers: 1,
                },
            );
            let uniforms: [u32; 4] = [0, 1.0f32.to_bits(), 0, 0];
            let bytes = uniforms
                .iter()
                .flat_map(|v| v.to_ne_bytes())
                .collect::<Vec<_>>();
            queue.write_buffer(&uploaded.uniform_buffer, 0, &bytes);
        }

        Vec::new()
    }

    fn paint(
        &self,
        _info: PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &CallbackResources,
    ) {
        let Some(resources) = callback_resources.get::<GpuEffectsResources>() else {
            return;
        };
        let Some(uploaded) = resources.uploaded_composites.get(&self.id) else {
            return;
        };
        render_pass.set_pipeline(&resources.blend_pipeline);
        render_pass.set_bind_group(0, &resources.transparent_base_bind_group, &[]);
        render_pass.set_bind_group(1, &uploaded.bind_group, &[]);
        render_pass.set_bind_group(2, &uploaded.uniform_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_gpu_effects_resources() {
        // Creating a wgpu device in a unit test is environment-dependent.
        // We just verify the module compiles and the function signature is correct.
        // A full test would require an async runtime and a valid GPU adapter.
        let _ = create_gpu_effects_resources;
        let callback = GpuCompositeCallback::new(1, [1, 1], vec![0, 0, 0, 0]);
        assert_eq!(callback.size, [1, 1]);
    }

    #[test]
    fn test_gpu_composite_callback_creation() {
        let callback = GpuCompositeCallback::new(42, [100, 200], vec![0; 100 * 200 * 4]);
        assert_eq!(callback.size, [100, 200]);
        assert_eq!(callback.rgba.len(), 100 * 200 * 4);
    }
}
