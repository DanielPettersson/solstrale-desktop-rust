use std::error::Error;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use eframe::egui::{ColorImage, Context, Image, TextureOptions, Vec2};
use solstrale::ray_trace;
use crate::{ErrorInfo, RenderControl, RenderInfo};
use crate::scene_model::{create_scene, Creator, SceneModel};

pub fn render_output(
    render_control: &mut RenderControl,
    render_info: &Arc<Mutex<RenderInfo>>,
    error_info: &mut ErrorInfo,
    scene_yaml: &str,
    render_size: Vec2,
    ctx: Context,
) -> Image {
    if render_control.abort_sender.is_none() && render_control.render_requested {

        let res = create_scene(scene_yaml).and_then(|scene_model| render(
            render_info.clone(),
            &scene_model,
            render_size,
            ctx.clone(),
        ));

        match res {
            Ok(abort_sender) => render_control.abort_sender = Some(abort_sender),
            Err(err) => {
                render_control.render_requested = false;
                error_info.show_error = true;
                error_info.error_message = format!("{}", err)
            }
        }
    }

    let render_info = render_info.lock().unwrap();
    Image::new(&render_info.texture_handle, render_size)
}

fn render(
    render_info: Arc<Mutex<RenderInfo>>,
    scene_model: &SceneModel,
    render_size: Vec2,
    ctx: Context,
) -> Result<Sender<bool>, Box<dyn Error>> {
    let (output_sender, output_receiver) = channel();
    let (abort_sender, abort_receiver) = channel();

    let scene = scene_model.create()?;

    thread::spawn(move || {
        ray_trace(
            render_size.x as u32,
            render_size.y as u32,
            scene,
            &output_sender,
            &abort_receiver,
        )
            .unwrap();
    });

    thread::spawn(move || {
        for render_output in output_receiver {
            let image = render_output.render_image;
            let fs = image.as_flat_samples();
            let color_image = ColorImage::from_rgb(
                [image.width() as usize, image.height() as usize],
                fs.as_slice(),
            );
            let mut render_info = render_info.lock().unwrap();
            render_info.progress = render_output.progress;
            render_info.texture_handle = ctx.load_texture(
                "render_texture",
                color_image,
                TextureOptions::default(),
            );

            ctx.request_repaint();
        }
    });

    Ok(abort_sender)
}
