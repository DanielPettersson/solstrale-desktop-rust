use image::RgbImage;
use solstrale::ray_trace;
use solstrale::renderer::RenderImageStrategy::OnlyFinal;
use solstrale_desktop_rust::model::{parse_scene_yaml, Creator, CreatorContext};
use std::error::Error;
use std::sync::mpsc::channel;
use std::{env, fs, thread};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let scene_path = args.get(1).ok_or("Failed to read scene path")?;
    let screen_width = args
        .get(2)
        .ok_or("Failed to read width")?
        .parse::<usize>()?;
    let screen_height = args
        .get(3)
        .ok_or("Failed to read height")?
        .parse::<usize>()?;
    let num_frames = args
        .get(4)
        .ok_or("Failed to read num_frames")?
        .parse::<usize>()?;

    for frame_index in 0..num_frames {
        let scene_yaml = fs::read_to_string(scene_path)?;

        let mut scene = parse_scene_yaml(&scene_yaml, frame_index)?.create(&CreatorContext {
            screen_width,
            screen_height,
        })?;
        scene.render_config.render_image_strategy = OnlyFinal;

        let (output_sender, output_receiver) = channel();
        let (_, abort_receiver) = channel();

        thread::spawn(move || {
            ray_trace(scene, &output_sender, &abort_receiver).unwrap();
        });

        let mut image = RgbImage::new(screen_width as u32, screen_height as u32);
        for render_output in output_receiver {
            if let Some(render_image) = render_output.render_image {
                image = render_image;
            }
        }

        image.save(format!("frame_{:0>8}.png", frame_index))?;
    }

    Ok(())
}
