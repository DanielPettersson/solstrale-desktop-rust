use std::error::Error;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::{fs, thread};

use clap::Parser;
use image::RgbImage;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use solstrale::ray_trace;
use solstrale::renderer::RenderImageStrategy::OnlyFinal;

use solstrale_desktop_rust::model::{parse_scene_yaml, Creator, CreatorContext};

#[derive(Parser)]
#[command(version, about, long_about = None, disable_help_flag = true)]
struct Cli {
    /// Path to the Solstrale scene description
    scene_path: PathBuf,

    /// Width of the rendered images
    #[arg(short, long, default_value_t = 800, value_parser = clap::value_parser!(u16).range(1..8000))]
    width: u16,

    /// Height of the rendered images
    #[arg(short, long, default_value_t = 600, value_parser = clap::value_parser!(u16).range(1..8000))]
    height: u16,

    /// Number of frames to render
    #[arg(short, long, default_value_t = 100, value_parser = clap::value_parser!(u16).range(1..))]
    num_frames: u16,

    #[clap(long, action = clap::ArgAction::HelpLong)]
    help: Option<bool>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let num_frames = cli.num_frames as usize;
    let screen_width = cli.width as usize;
    let screen_height = cli.height as usize;
    let scene_path = cli.scene_path;

    let multi_progress = MultiProgress::new();
    let total_progress_style = ProgressStyle::with_template(
        "[elapsed: {elapsed}, eta: {eta}] {wide_bar:.green/blue} {percent:>3}% Total progress",
    )
    .unwrap()
    .progress_chars("##-");

    let frame_progress_style =
        ProgressStyle::with_template("{wide_bar:.green/blue} {percent:>3}% Frame progress")
            .unwrap()
            .progress_chars("##-");

    let total_progress_bar =
        multi_progress.add(ProgressBar::new(num_frames as u64).with_style(total_progress_style));

    for frame_index in 0..num_frames {
        let scene_yaml = fs::read_to_string(scene_path.clone())?;

        let mut scene = parse_scene_yaml(&scene_yaml, frame_index)?.create(&CreatorContext {
            screen_width,
            screen_height,
        })?;
        scene.render_config.render_image_strategy = OnlyFinal;

        let samples_per_pixel = scene.render_config.samples_per_pixel as u64;
        total_progress_bar.set_length(num_frames as u64 * samples_per_pixel);
        let frame_progress_bar = multi_progress
            .add(ProgressBar::new(samples_per_pixel).with_style(frame_progress_style.clone()));

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
            total_progress_bar.inc(1);
            frame_progress_bar.inc(1);
        }

        image.save(format!("frame_{:0>8}.png", frame_index))?;
        multi_progress.remove(&frame_progress_bar);
    }

    multi_progress.clear().unwrap();
    Ok(())
}
