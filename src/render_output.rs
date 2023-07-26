use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

use eframe::egui::{Color32, ColorImage, Context, Image, TextureOptions, Vec2};
use solstrale::ray_trace;

use crate::scene_model::{create_scene, Creator};
use crate::{ErrorInfo, RenderControl, RenderMessage, RenderedImage};

pub fn render_output(
    render_control: &mut RenderControl,
    rendered_image: &mut RenderedImage,
    error_info: &mut ErrorInfo,
    scene_yaml: &str,
    render_size: Vec2,
    ctx: Context,
) -> Option<Image> {
    if render_control.render_requested {
        if let Some(sender) = &render_control.abort_sender {
            sender.send(true).ok();
        }
    }

    if render_control.abort_sender.is_none() && render_control.render_requested {
        let res = render(scene_yaml, render_size, ctx.clone());
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
                ColorImage::new([1, 1], Color32::BLACK),
                TextureOptions::default(),
            )
        });

        Some(Image::new(texture_handle, render_size))
    }
}

fn render(
    scene_yaml: &str,
    render_size: Vec2,
    ctx: Context,
) -> (Receiver<RenderMessage>, Sender<bool>) {
    let (output_sender, output_receiver) = channel();
    let (abort_sender, abort_receiver) = channel();
    let (render_sender, render_receiver) = channel();

    let render_sender_clone = render_sender.clone();
    let scene_yaml_str = scene_yaml.to_string();

    thread::spawn(move || {

        // Not too fancy, solution. But adding a sleep here avoids flickering
        // when restarting rendering with a scene that loads really fast
        thread::sleep(Duration::from_millis(300));

        let res = (|| match create_scene(&scene_yaml_str)?.create() {
            Ok(scene) => ray_trace(
                render_size.x as u32,
                render_size.y as u32,
                scene,
                &output_sender,
                &abort_receiver,
            ),
            Err(err) => Err(err),
        })();

        if let Err(err) = res {
            render_sender_clone
                .send(RenderMessage::Error(format!("{}", err)))
                .unwrap();
        };
    });

    thread::spawn(move || {
        for render_output in output_receiver {
            render_sender
                .send(RenderMessage::SampleRendered(render_output))
                .unwrap();
            ctx.request_repaint();
        }
    });

    (render_receiver, abort_sender)
}
