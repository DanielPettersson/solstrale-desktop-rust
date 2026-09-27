use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui::{Context, PointerButton, Sense, Ui, Vec2};
use eframe::wgpu;
use eframe::wgpu::util::DeviceExt;
use solstrale::camera::CameraConfig;
use solstrale::geo::vec3::Vec3;
use solstrale::ray_trace;
use solstrale::renderer::SceneUpdate;
use solstrale::util::tone_map::ToneMapper;

use crate::model::orbit_camera::{CameraSnapshot, OrbitCamera, ViewAction, view_action};
use crate::model::scene::Scene;
use crate::model::scope::Scope;
use crate::model::{Creator, CreatorContext};
use crate::render_scheduler::{Inputs, create_update};
use crate::{
    Build, Built, DISPLAY_TONE_MAPPER, RenderCallback, RenderControl, RenderError, RenderMessage,
    RenderResources, RenderedImage, UpdateRequest,
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

// Of the rendered image, which is stretched over the viewport
@group(0) @binding(0) var<uniform> image_size: vec2<f32>;
@group(0) @binding(1) var<storage, read> buffer: array<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let x = min(u32(in.uv.x * image_size.x), u32(image_size.x) - 1u);
    let y = min(u32(in.uv.y * image_size.y), u32(image_size.y) - 1u);
    let index = (y * u32(image_size.x) + x) * 4u;

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

    let image_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Image Size Buffer"),
        contents: bytemuck::cast_slice(&[0.0f32, 0.0f32]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    RenderResources {
        pipeline,
        bind_group_layout,
        image_size_buffer,
        target_format,
        device: device.clone(),
        queue: queue.clone(),
    }
}

pub fn render_output(
    ui: &mut Ui,
    render_control: &mut RenderControl,
    rendered_image: &mut RenderedImage,
    scene: Option<&Scene>,
    frame_index: usize,
    viewport_size: Vec2,
) {
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
                render_control.view_dragged = true;
                ui.ctx().request_repaint();
            }
        }
    }

    if let (Some(scene), Some(resources)) = (scene, rendered_image.render_resources.clone()) {
        dispatch(
            render_control,
            scene,
            frame_index,
            viewport_size,
            ui.ctx(),
            &resources,
        );
    }
}

/// Sends the view to the running render, and starts what
/// [`RenderControl::schedule`] asked for: a new render, or an update to build.
pub fn dispatch(
    render_control: &mut RenderControl,
    scene: &Scene,
    frame_index: usize,
    viewport_size: Vec2,
    ctx: &Context,
    resources: &Arc<RenderResources>,
) {
    if render_control.camera_updated
        && let (Some(sender), Some(orbit_camera)) =
            (&render_control.update_sender, &render_control.orbit_camera)
        && sender.send(CameraConfig::from(orbit_camera).into()).is_ok()
    {
        render_control.camera_updated = false;
        render_control.edit_timer.sent();
    }

    if render_control.render_requested {
        if let Some(sender) = &render_control.abort_sender {
            sender.send(true).ok();
        }
        render_control.end_render();
    }

    let screen = (viewport_size.x as usize, viewport_size.y as usize);
    if render_control.render_requested && screen.0 > 0 && screen.1 > 0 {
        match scene.camera_at(frame_index) {
            Ok(camera) => {
                let snapshot = CameraSnapshot::from(&camera);
                if view_action(
                    render_control.applied_camera.as_ref(),
                    &snapshot,
                    render_control.orbit_camera.is_some(),
                    render_control.reset_view,
                ) == ViewAction::Reset
                {
                    render_control.orbit_camera = Some(OrbitCamera::from_config(&camera, 1.));
                    render_control.applied_camera = Some(snapshot);
                }
            }
            // The render thread reports the error when it evaluates the same
            Err(_) => {
                render_control.orbit_camera = None;
                render_control.applied_camera = None;
            }
        }
        render_control.reset_view = false;

        let res = render(
            scene.clone(),
            frame_index,
            render_control
                .orbit_camera
                .as_ref()
                .map(|o| CameraSnapshot::from(&CameraConfig::from(o))),
            screen,
            ctx,
            resources.clone(),
        );
        render_control.render_receiver = Some(res.0);
        render_control.abort_sender = Some(res.1);
        render_control.update_sender = Some(res.2);
        render_control.render_requested = false;
        render_control.edit_pending = false;
        render_control.loading_scene = true;
        render_control.camera_updated = false;
        render_control.overlay = render_control.overlay_next_render;
        render_control.overlay_next_render = false;
        render_control.render_error = None;
        render_control.rendered = Inputs::of(scene, frame_index, screen).ok();
        render_control.edit_timer.new_render();
    }

    if let Some(request) = render_control.update_requested.take() {
        match &render_control.update_sender {
            Some(sender) => {
                let inputs = request.inputs.clone();
                render_control.build = Some(Build {
                    receiver: build_update(request, sender.clone(), ctx, resources.clone()),
                    inputs,
                    started: Instant::now(),
                });
            }
            // The render ended since, so the edit starts a new one
            None => render_control.edit_pending = true,
        }
    }
}

/// Builds the scene and ray traces it on a thread. `camera` replaces the
/// scene's own camera, so a restart keeps the view the user has orbited to.
fn render(
    scene: Scene,
    frame_index: usize,
    camera: Option<CameraSnapshot>,
    screen: (usize, usize),
    ctx: &Context,
    resources: Arc<RenderResources>,
) -> (Receiver<RenderMessage>, Sender<bool>, Sender<SceneUpdate>) {
    let (output_sender, output_receiver) = channel();
    let (abort_sender, abort_receiver) = channel();
    let (update_sender, update_receiver) = channel();
    let (render_sender, render_receiver) = channel();

    let render_sender_clone = render_sender.clone();
    let ctx1 = ctx.clone();
    let ctx2 = ctx.clone();

    thread::spawn(move || {
        let res = (|| {
            let mut scene = scene.create(&CreatorContext {
                screen_width: screen.0,
                screen_height: screen.1,
                device: &resources.device,
                queue: &resources.queue,
                scope: &Scope::builtin(frame_index),
                refit_models: false,
            })?;
            if let Some(camera) = &camera {
                scene.camera = camera.into();
            }

            ray_trace(
                scene,
                &output_sender,
                &update_receiver,
                &abort_receiver,
                &resources.device,
                &resources.queue,
                true,
            )
        })();

        if let Err(err) = res {
            render_sender_clone
                .send(RenderMessage::Error(RenderError::new(&*err)))
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

    (render_receiver, abort_sender, update_sender)
}

/// Builds the parts of the scene that changed on a thread, and sends them to
/// the running render
fn build_update(
    request: UpdateRequest,
    updates: Sender<SceneUpdate>,
    ctx: &Context,
    resources: Arc<RenderResources>,
) -> Receiver<Built> {
    let (sender, receiver) = channel();
    let ctx = ctx.clone();
    thread::spawn(move || {
        let (update, parts, error) = create_update(
            &request.scene,
            &CreatorContext {
                screen_width: request.screen.0,
                screen_height: request.screen.1,
                device: &resources.device,
                queue: &resources.queue,
                scope: &Scope::builtin(request.frame),
                refit_models: true,
            },
            request.parts,
        );
        if !parts.is_empty() {
            updates.send(update).ok();
        }
        sender
            .send(Built {
                parts,
                error: error.map(|e| RenderError::new(&*e)),
            })
            .ok();
        ctx.request_repaint();
    });
    receiver
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
