use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread;

use eframe::egui::load::SizedTexture;
use eframe::egui::{Color32, ColorImage, Context, Image, TextureOptions, Vec2};
use solstrale::ray_trace;

use crate::model::{parse_scene_yaml, Creator, CreatorContext};
use crate::{ErrorInfo, RenderControl, RenderMessage, RenderedImage};

pub fn render_output<'a>(
    render_control: &mut RenderControl,
    rendered_image: &mut RenderedImage,
    error_info: &mut ErrorInfo,
    scene_yaml: &str,
    render_size: Vec2,
    ctx: &Context,
) -> Option<Image<'a>> {
    if render_control.render_requested {
        if let Some(sender) = &render_control.abort_sender {
            sender.send(true).ok();
        }
    }

    if render_control.abort_sender.is_none() && render_control.render_requested {
        rendered_image.num_pixels = render_size.x as u32 * render_size.y as u32;

        let res = render(scene_yaml, render_size, ctx);
        render_control.render_receiver = Some(res.0);
        render_control.abort_sender = Some(res.1);
        render_control.render_requested = false;
        render_control.loading_scene = true;
    }

    if let Some(render_receiver) = &render_control.render_receiver {
        match render_receiver.try_recv() {
            Ok(render_message) => match render_message {
                RenderMessage::SampleRendered(render_progress) => {
                    if let Some(image) = render_progress.render_image {
                        let fs = image.as_flat_samples();
                        let color_image = ColorImage::from_rgb(
                            [image.width() as usize, image.height() as usize],
                            fs.as_slice(),
                        );
                        rendered_image.num_pixels = image.width() * image.height();
                        rendered_image.rgb_image = Some(image);
                        match rendered_image.texture_handle.as_mut() {
                            None => {
                                rendered_image.texture_handle = Some(ctx.load_texture(
                                    "render_texture",
                                    color_image,
                                    TextureOptions::default(),
                                ))
                            }
                            Some(handle) => handle.set(color_image, TextureOptions::default()),
                        };
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
                ColorImage::new(
                    [render_size.x as usize, render_size.y as usize],
                    Color32::BLACK,
                ),
                TextureOptions::default(),
            )
        });

        Some(Image::from_texture(SizedTexture::new(
            texture_handle,
            render_size,
        )))
    }
}

fn render(
    scene_yaml: &str,
    render_size: Vec2,
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
            screen_width: render_size.x as usize,
            screen_height: render_size.y as usize,
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
