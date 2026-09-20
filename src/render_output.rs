use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui::{Context, PointerButton, Sense, Ui, Vec2};
use eframe::wgpu;
use eframe::wgpu::util::DeviceExt;
use solstrale::geo::vec3::Vec3;
use solstrale::ray_trace;
use solstrale::util::tone_map::ToneMapper;

use crate::model::orbit_camera::OrbitCamera;
use crate::model::scene::Scene;
use crate::model::{Creator, CreatorContext, parse_scene_yaml};
use crate::{
    DISPLAY_TONE_MAPPER, ErrorInfo, RenderCallback, RenderControl, RenderMessage, RenderResources,
    RenderedImage,
};

/// Repaints are asked for at most this often. Progress messages can arrive
/// faster than the screen can show them.
const FRAME_INTERVAL: Duration = Duration::from_millis(16);

/// Scroll delta to relative zoom amount. One wheel notch is ~50 units.
const ZOOM_SENSITIVITY: f64 = 0.005;

/// The blit shader, less the tone mapping function that gets spliced in by
/// [`shader_source`].
///
/// `fs_main` returns the tone-mapped value without applying a transfer
/// function, because the surface it draws to is an sRGB format and the
/// hardware encodes on write. That is the same encode `buffer_to_image` applies
/// when an image is saved -- it used to be gamma 2.0 there against sRGB's ~2.2
/// here -- so the two display paths now share the tone curve and the transfer
/// function exactly. What is left between them is quantisation: the hardware
/// rounds to nearest over 255, the readback truncates over 256, which is worth
/// at most a code value.
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

    return vec4<f32>(solstrale_tone_map(vec3<f32>(r, g, b)), 1.0);
}
"#;

/// The blit shader with the tone mapping curve prepended.
///
/// The curve comes from the library rather than being written again here, so
/// the viewport and a saved image cannot disagree about what the render looks
/// like. WGSL has no include directive, so this is string concatenation.
fn shader_source(tone_mapper: ToneMapper) -> String {
    format!("{}\n{}", tone_mapper.wgsl(), SHADER)
}

pub fn create_render_resources(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    target_format: wgpu::TextureFormat,
) -> RenderResources {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Render Shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source(DISPLAY_TONE_MAPPER).into()),
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
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
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
        multiview_mask: None,
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
    // Process messages from the renderer
    if let Some(render_receiver) = &render_control.render_receiver {
        loop {
            match render_receiver.try_recv() {
                Ok(render_message) => match render_message {
                    RenderMessage::SampleRendered(render_progress) => {
                        // The renderer reports the same buffer for the whole
                        // run. Only swapping the handle when it really changed
                        // keeps the cached blit bind group valid.
                        if rendered_image.output_buffer.as_ref()
                            != Some(&render_progress.output_buffer)
                        {
                            rendered_image.output_buffer = Some(render_progress.output_buffer);
                        }
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

    // UI and Interaction
    if viewport_size.x > 0.0 && viewport_size.y > 0.0 {
        let (rect, response) = ui.allocate_exact_size(viewport_size, Sense::drag());

        // Paint the last rendered image
        let has_image = rendered_image
            .output_buffer
            .as_ref()
            .is_some_and(|output_buffer| output_buffer.size() > 0);

        if has_image
            && let Some(resources) = rendered_image.render_resources.clone()
            && let Some(bind_group) = rendered_image.blit_bind_group()
        {
            ui.painter()
                .add(eframe::egui_wgpu::Callback::new_paint_callback(
                    rect,
                    RenderCallback {
                        resources,
                        bind_group,
                        width: rendered_image.width,
                        height: rendered_image.height,
                    },
                ));
        }

        // Handle camera interactions
        if let Some(orbit_camera) = &mut render_control.orbit_camera {
            let mut input_changed = false;
            if response.dragged_by(PointerButton::Primary) {
                let delta = response.drag_delta();
                if delta.x != 0.0 || delta.y != 0.0 {
                    orbit_camera.orbit(-delta.x as f64 * 0.01, -delta.y as f64 * 0.01);
                    input_changed = true;
                }
            }
            if response.dragged_by(PointerButton::Secondary)
                || response.dragged_by(PointerButton::Middle)
            {
                let delta = response.drag_delta();
                if delta.x != 0.0 || delta.y != 0.0 {
                    orbit_camera.pan(
                        delta.x as f64 * 0.001 * orbit_camera.current_distance,
                        delta.y as f64 * 0.001 * orbit_camera.current_distance,
                        Vec3::new(0., 1., 0.),
                    );
                    input_changed = true;
                }
            }
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                orbit_camera.zoom(scroll as f64 * ZOOM_SENSITIVITY);
                input_changed = true;
            }

            if orbit_camera.update() || input_changed {
                render_control.camera_updated = true;
                ui.ctx().request_repaint();
            }
        }
    }

    // Handle render restarts

    if render_control.camera_updated
        && let (Some(sender), Some(orbit_camera)) = (
            &render_control.camera_config_sender,
            &render_control.orbit_camera,
        )
        && sender.send(orbit_camera.into()).is_ok()
    {
        render_control.camera_updated = false;
    }

    if render_control.render_requested {
        if let Some(sender) = &render_control.abort_sender {
            sender.send(true).ok();
        }

        render_control.abort_sender = None;
        render_control.render_receiver = None;
        render_control.camera_config_sender = None;

        if !render_control.camera_updated {
            render_control.scene = None;
            render_control.orbit_camera = None;
        }
    }

    if render_control.render_requested
        && viewport_size.x > 0.0
        && viewport_size.y > 0.0
        && let Some(resources) = rendered_image.render_resources.as_ref()
    {
        if render_control.scene.is_none()
            && let Ok(s) = parse_scene_yaml(scene_yaml, 0)
        {
            let ctx = CreatorContext {
                screen_width: viewport_size.x as usize,
                screen_height: viewport_size.y as usize,
                device: &resources.device,
                queue: &resources.queue,
            };

            render_control.orbit_camera = Some(OrbitCamera::new(&s.camera, &ctx, 1.));
            render_control.scene = Some(s);
        }

        if let (Some(scene), Some(orbit_camera)) =
            (&mut render_control.scene, &render_control.orbit_camera)
        {
            scene.camera.look_from = orbit_camera.look_from().into();
            scene.camera.look_at = Some(orbit_camera.look_at().into());
        }

        let res = render(
            scene_yaml,
            render_control.scene.clone(),
            viewport_size,
            ui.ctx(),
            resources.clone(),
        );
        rendered_image.width = viewport_size.x as u32;
        rendered_image.height = viewport_size.y as u32;
        render_control.render_receiver = Some(res.0);
        render_control.abort_sender = Some(res.1);
        render_control.camera_config_sender = Some(res.2);
        render_control.render_requested = false;
        if !render_control.camera_updated {
            render_control.loading_scene = true;
        }
        render_control.camera_updated = false;
    }
}

fn render(
    scene_yaml: &str,
    scene: Option<Scene>,
    viewport_size: Vec2,
    ctx: &Context,
    resources: Arc<RenderResources>,
) -> (
    Receiver<RenderMessage>,
    Sender<bool>,
    Sender<solstrale::camera::CameraConfig>,
) {
    let (output_sender, output_receiver) = channel();
    let (abort_sender, abort_receiver) = channel();
    let (camera_config_sender, camera_config_receiver) = channel();
    let (render_sender, render_receiver) = channel();

    if viewport_size.x <= 0.0 || viewport_size.y <= 0.0 {
        return (render_receiver, abort_sender, camera_config_sender);
    }

    let render_sender_clone = render_sender.clone();
    let scene_yaml_str = scene_yaml.to_string();
    let ctx1 = ctx.clone();
    let ctx2 = ctx.clone();

    thread::spawn(move || {
        let res = (|| {
            let scene = match scene {
                Some(s) => s,
                None => parse_scene_yaml(&scene_yaml_str, 0)?,
            }
            .create(&CreatorContext {
                screen_width: viewport_size.x as usize,
                screen_height: viewport_size.y as usize,
                device: &resources.device,
                queue: &resources.queue,
            })?;

            ray_trace(
                scene,
                &output_sender,
                &camera_config_receiver,
                &abort_receiver,
                &resources.device,
                &resources.queue,
                true,
            )
        })();

        if let Err(err) = res {
            let mut err_msg = format!("{}", err);
            if let Some(s) = err.source() {
                err_msg = err_msg + &format!("\n{}", s);
            }

            render_sender_clone
                .send(RenderMessage::Error(err_msg))
                .unwrap_or(());
            ctx1.request_repaint();
        };
    });

    thread::spawn(move || {
        let mut last_repaint: Option<Instant> = None;

        for render_output in output_receiver {
            render_sender
                .send(RenderMessage::SampleRendered(render_output))
                .unwrap_or(());

            let now = Instant::now();
            match last_repaint {
                Some(last) if now.duration_since(last) < FRAME_INTERVAL => {
                    // Too soon to be worth a frame, but make sure the progress
                    // we just sent is not left sitting unpainted.
                    ctx2.request_repaint_after(FRAME_INTERVAL - now.duration_since(last));
                }
                _ => {
                    last_repaint = Some(now);
                    ctx2.request_repaint();
                }
            }
        }
    });

    (render_receiver, abort_sender, camera_config_sender)
}

#[cfg(test)]
mod tests {
    use super::*;
    use solstrale::util::wgpu_util::get_wgpu_device_and_queue;

    /// The blit shader is assembled from two strings at runtime, so nothing in
    /// the Rust build checks it: a mistake in the splice, or a curve whose
    /// emitted WGSL does not compile, would first show up as a blank window.
    /// Compiling every curve here turns that into a test failure.
    #[test]
    fn every_tone_mapper_produces_a_shader_that_compiles() {
        let (device, _) = get_wgpu_device_and_queue();

        for mapper in [
            ToneMapper::Aces,
            ToneMapper::PbrNeutral,
            ToneMapper::Reinhard { white_point: 4. },
            ToneMapper::Clamp,
        ] {
            let source = shader_source(mapper);
            assert!(
                source.contains("solstrale_tone_map"),
                "{:?} emitted no tone map function",
                mapper
            );

            let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
            let _ = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Tone Map Shader Test"),
                source: wgpu::ShaderSource::Wgsl(source.as_str().into()),
            });
            let err = pollster::block_on(error_scope.pop());
            assert!(err.is_none(), "{:?} failed to compile: {:?}", mapper, err);
        }
    }
}
