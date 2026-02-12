use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;

use eframe::egui::{Context, Sense, Ui, Vec2};
use eframe::wgpu;
use eframe::wgpu::util::DeviceExt;
use solstrale::ray_trace;

use crate::model::{parse_scene_yaml, Creator, CreatorContext};
use crate::{
    ErrorInfo, RenderCallback, RenderControl, RenderMessage, RenderResources, RenderedImage,
};

const SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vec2<f32>(f32((in_vertex_index << 1u) & 2u), f32(in_vertex_index & 2u));
    out.position = vec4<f32>(out.uv * 2.0 - 1.0, 0.0, 1.0);
    out.uv.y = 1.0 - out.uv.y;
    return out;
}

@group(0) @binding(0) var<uniform> viewport_size: vec2<f32>;
@group(0) @binding(1) var<storage, read> buffer: array<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let x = min(u32(in.uv.x * viewport_size.x), u32(viewport_size.x) - 1u);
    let y = min(u32(in.uv.y * viewport_size.y), u32(viewport_size.y) - 1u);
    let index = (y * u32(viewport_size.x) + x) * 4u;

    let r = buffer[index];
    let g = buffer[index + 1u];
    let b = buffer[index + 2u];

    return vec4<f32>(r, g, b, 1.0);
}
"#;

pub fn create_render_resources(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    target_format: wgpu::TextureFormat,
) -> RenderResources {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Render Shader"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Render Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    let viewport_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Viewport Size Buffer"),
        contents: bytemuck::cast_slice(&[0.0f32, 0.0f32]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    RenderResources {
        pipeline,
        bind_group_layout,
        viewport_size_buffer,
        target_format,
        device: device.clone(),
        queue: queue.clone(),
    }
}

pub fn render_output(
    ui: &mut Ui,
    render_control: &mut RenderControl,
    rendered_image: &mut RenderedImage,
    error_info: &mut ErrorInfo,
    scene_yaml: &str,
    viewport_size: Vec2,
) {
    let ctx = ui.ctx();

    if render_control.render_requested {
        if let Some(sender) = &render_control.abort_sender {
            sender.send(true).ok();
        }
    }

    if render_control.abort_sender.is_none()
        && render_control.render_requested
        && viewport_size.x > 0.0
        && viewport_size.y > 0.0
    {
        if let Some(resources) = rendered_image.render_resources.as_ref() {
            let res = render(scene_yaml, viewport_size, ctx, resources.clone());
            rendered_image.width = viewport_size.x as u32;
            rendered_image.height = viewport_size.y as u32;
            render_control.render_receiver = Some(res.0);
            render_control.abort_sender = Some(res.1);
            render_control.render_requested = false;
            render_control.loading_scene = true;
        }
    }

    if let Some(render_receiver) = &render_control.render_receiver {
        loop {
            match render_receiver.try_recv() {
                Ok(render_message) => match render_message {
                    RenderMessage::SampleRendered(render_progress) => {
                        rendered_image.output_buffer = Some(Arc::new(render_progress.output_buffer));
                        rendered_image.progress = render_progress.progress;
                        if let Some(fps) = render_progress.fps {
                            rendered_image.fps = fps;
                        }
                        rendered_image.estimated_time_left = render_progress.estimated_time_left;
                        render_control.loading_scene = false;
                    }
                    RenderMessage::Error(error_message) => {
                        error_info.handle_str(&error_message);
                        render_control.loading_scene = false;
                    }
                },
                Err(err) => {
                    match err {
                        TryRecvError::Empty => {}
                        TryRecvError::Disconnected => {
                            render_control.abort_sender = None;
                        }
                    }
                    break;
                }
            }
        }
    }

    if !render_control.loading_scene
        && !render_control.render_requested
        && viewport_size.x > 0.0
        && viewport_size.y > 0.0
    {
        let (rect, _response) = ui.allocate_exact_size(viewport_size, Sense::hover());

        if let (Some(resources), Some(output_buffer)) = (
            &rendered_image.render_resources,
            &rendered_image.output_buffer,
        ) {
            if output_buffer.size() > 0 {
                ui.painter()
                    .add(eframe::egui_wgpu::Callback::new_paint_callback(
                        rect,
                        RenderCallback {
                            resources: resources.clone(),
                            output_buffer: output_buffer.clone(),
                            width: rendered_image.width,
                            height: rendered_image.height,
                            bind_group: Arc::new(Mutex::new(None)),
                        },
                    ));
            }
        }
    }
}

fn render(
    scene_yaml: &str,
    viewport_size: Vec2,
    ctx: &Context,
    resources: Arc<RenderResources>,
) -> (Receiver<RenderMessage>, Sender<bool>) {
    let (output_sender, output_receiver) = channel();
    let (abort_sender, abort_receiver) = channel();
    let (render_sender, render_receiver) = channel();

    if viewport_size.x <= 0.0 || viewport_size.y <= 0.0 {
        return (render_receiver, abort_sender);
    }

    let render_sender_clone = render_sender.clone();
    let scene_yaml_str = scene_yaml.to_string();
    let ctx1 = ctx.clone();
    let ctx2 = ctx.clone();

    thread::spawn(move || {
        let res = (|| {
            let scene = parse_scene_yaml(&scene_yaml_str, 0)?.create(&CreatorContext {
                screen_width: viewport_size.x as usize,
                screen_height: viewport_size.y as usize,
                device: &resources.device,
                queue: &resources.queue,
            })?;

            ray_trace(
                scene,
                &output_sender,
                &abort_receiver,
                &resources.device,
                &resources.queue,
            )
        })();

        if let Err(err) = res {
            let mut err_msg = format!("{}", err);
            if let Some(s) = err.source() {
                err_msg = err_msg + &format!("\n{}", s);
            }

            render_sender_clone
                .send(RenderMessage::Error(err_msg))
                .unwrap();
            ctx1.request_repaint();
        };
    });

    thread::spawn(move || {
        for render_output in output_receiver {
            render_sender
                .send(RenderMessage::SampleRendered(render_output))
                .unwrap();
            ctx2.request_repaint();
        }
    });

    (render_receiver, abort_sender)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_presence() {
        assert!(!SHADER.is_empty());
        assert!(SHADER.contains("@fragment"));
        assert!(SHADER.contains("@vertex"));
    }

    #[test]
    fn test_render_resources_presence() {
        // This test just ensures the type exists and can be referenced
        let _: Option<RenderResources> = None;
    }
}
