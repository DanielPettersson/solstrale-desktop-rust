use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread;

use eframe::egui::load::SizedTexture;
use eframe::egui::{Color32, ColorImage, Context, Image, TextureOptions, Vec2};
use eframe::wgpu;
use solstrale::ray_trace;

use crate::model::{parse_scene_yaml, Creator, CreatorContext};
use crate::{ErrorInfo, RenderControl, RenderMessage, RenderedImage, RenderResources};

const SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var<uniform> viewport_size: vec2<f32>;
@group(0) @binding(1) var<storage, read> buffer: array<f32>;

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    let x = u32(in.uv.x * viewport_size.x);
    let y = u32(in.uv.y * viewport_size.y);
    let index = (y * u32(viewport_size.x) + x) * 3u;

    let r = buffer[index];
    let g = buffer[index + 1u];
    let b = buffer[index + 2u];

    return vec4<f32>(r, g, b, 1.0);
}
"#;

pub fn render_output<'a>(
    render_control: &mut RenderControl,
    rendered_image: &mut RenderedImage,
    error_info: &mut ErrorInfo,
    scene_yaml: &str,
    viewport_size: Vec2,
    ctx: &Context,
) -> Option<Image<'a>> {
    if render_control.render_requested {
        if let Some(sender) = &render_control.abort_sender {
            sender.send(true).ok();
        }
    }

    if render_control.abort_sender.is_none() && render_control.render_requested {
        let res = render(scene_yaml, viewport_size, ctx);
        rendered_image.texture_handle = None;
        rendered_image.width = viewport_size.x as u32;
        rendered_image.height = viewport_size.y as u32;
        render_control.render_receiver = Some(res.0);
        render_control.abort_sender = Some(res.1);
        render_control.render_requested = false;
        render_control.loading_scene = true;
    }

    if let Some(render_receiver) = &render_control.render_receiver {
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
            Err(err) => match err {
                TryRecvError::Empty => {}
                TryRecvError::Disconnected => {
                    render_control.abort_sender = None;
                }
            },
        }
    }

    if render_control.loading_scene || render_control.render_requested {
        None
    } else {
        let texture_handle = rendered_image.texture_handle.get_or_insert_with(|| {
            ctx.load_texture(
                "",
                ColorImage::new([1, 1], vec![Color32::BLACK]),
                TextureOptions::default(),
            )
        });

        Some(Image::from_texture(SizedTexture::new(
            texture_handle,
            Vec2::new(rendered_image.width as f32, rendered_image.height as f32),
        )))
    }
}

fn render(
    scene_yaml: &str,
    viewport_size: Vec2,
    ctx: &Context,
) -> (Receiver<RenderMessage>, Sender<bool>) {
    let (output_sender, output_receiver) = channel();
    let (abort_sender, abort_receiver) = channel();
    let (render_sender, render_receiver) = channel();

    let render_sender_clone = render_sender.clone();
    let scene_yaml_str = scene_yaml.to_string();
    let ctx1 = ctx.clone();
    let ctx2 = ctx.clone();

    thread::spawn(move || {
        let res = (|| match parse_scene_yaml(&scene_yaml_str, 0)?.create(&CreatorContext {
            screen_width: viewport_size.x as usize,
            screen_height: viewport_size.y as usize,
        }) {
            Ok(scene) => ray_trace(scene, &output_sender, &abort_receiver),
            Err(err) => Err(err),
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
    }

    #[test]
    fn test_render_resources_presence() {
        // This test just ensures the type exists and can be referenced
        let _ : Option<RenderResources> = None;
    }
}
