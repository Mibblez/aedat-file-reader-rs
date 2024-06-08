use std::io::Error;
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use image::{ImageBuffer, Rgb};
use opencv::core::Size;
use opencv::imgcodecs::{imread, IMREAD_COLOR};

use opencv::prelude::*;
use opencv::videoio::VideoWriter;

use crate::aedat_data::{CameraParameters, Event};
use crate::cli_configs::VidConfig;

mod colors {
    pub static RED: [u8; 3] = [255u8, 0u8, 0u8];
    pub static GREEN: [u8; 3] = [0u8, 255u8, 0u8];
    pub static BLACK: [u8; 3] = [0u8, 0u8, 0u8];
}

pub struct Frame {
    pub img: ImageBuffer<Rgb<u8>, Vec<u8>>,
    pub count: usize,
}

impl Frame {
    pub fn save_frame(&self, frame_tmp_dir: &Path, filename: &str) -> std::io::Result<()> {
        let result = self.img.save(format!(
            "{}/{}_frame{}.png",
            frame_tmp_dir.to_string_lossy(),
            filename,
            self.count
        ));

        if let Ok(()) = result {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::Other, "Could not save frame"))
        }

        // Ok(())
    }
}

fn prep_frame_tmp_dir(tmp_dir: &PathBuf) -> std::io::Result<()> {
    // Create frame tmp directory if it does not exist
    if let Err(e) = fs::create_dir(tmp_dir) {
        if e.kind() != ErrorKind::AlreadyExists {
            return Err(e);
        }
    }

    // Clear any old files
    let paths = fs::read_dir(tmp_dir)?;
    for path in paths {
        fs::remove_file(path?.path())?;
    }

    Ok(())
}

pub fn create_time_based_video(
    events: Vec<Event>,
    config: &VidConfig,
    cam: &CameraParameters,
) -> std::io::Result<()> {
    let frame_tmp_dir = if config.keep_frames {
        config.filename.clone()
    } else {
        PathBuf::from(".frames_tmp")
    };
    let video_name = Path::new(&config.filename)
        .file_stem()
        .unwrap()
        .to_string_lossy();

    prep_frame_tmp_dir(&frame_tmp_dir)?;

    let on_color = image::Rgb(colors::GREEN);
    let off_color = image::Rgb(colors::RED);
    let black = image::Rgb(colors::BLACK);

    const BUF_SIZE: usize = 150;
    let mut write_buf: Vec<Frame> = Vec::with_capacity(BUF_SIZE);

    // Init canvas
    let mut img = ImageBuffer::from_fn(u32::from(cam.camera_x), u32::from(cam.camera_y), |_, _| {
        image::Rgb(colors::BLACK)
    });

    // Define end time relative to the first event
    let mut end_time: i32 = match events.first() {
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "No events exist",
            ))
        }
        Some(event) => event.get_timestamp() + config.window_size as i32,
    };
    let mut frames_created = 0;

    for event in events {
        // Place a pixel on the image canvas with the appropriate color & position
        place_pixel(config, cam, on_color, off_color, &mut img, &event);

        if event.get_timestamp() > end_time {
            frames_created += 1;
            if frames_created == config.max_frames {
                break;
            }

            end_time = event.get_timestamp() + config.window_size as i32;

            let count = fs::read_dir(&frame_tmp_dir)?.count() + write_buf.len();

            // Save image to buffer
            write_buf.push(Frame {
                img: img.clone(),
                count,
            });

            // Write all frames to disk once enough have been saved to buffer
            if write_buf.len() == BUF_SIZE {
                for frame in &write_buf {
                    frame.save_frame(&frame_tmp_dir, &video_name)?;
                }
                write_buf.clear();
            }

            // Reset canvas to black
            for pixel in img.pixels_mut() {
                *pixel = black;
            }
        }
    }

    // Save any remaining frames in buffer
    for frame in &write_buf {
        frame.save_frame(&frame_tmp_dir, &video_name)?;
    }

    // Save any remaining events in current working img
    let count = std::fs::read_dir(&frame_tmp_dir)?.count();
    img.save(format!(
        "{}/{}_tmp_{}.png",
        frame_tmp_dir.display(),
        video_name,
        count
    ))
    .unwrap();

    if !config.omit_video {
        //encode_frames(&video_name, &frame_tmp_dir)?;
        encode_frames(&config.filename.to_string_lossy(), &frame_tmp_dir, cam)?;
    }

    if !config.keep_frames {
        prep_frame_tmp_dir(&frame_tmp_dir)?;
    }

    Ok(())
}

pub fn create_event_based_video(
    events: Vec<Event>,
    config: &VidConfig,
    cam: &CameraParameters,
) -> std::io::Result<()> {
    let frame_tmp_dir = if config.keep_frames {
        config.filename.clone()
    } else {
        PathBuf::from(".frames_tmp")
    };
    let video_name = Path::new(&config.filename)
        .file_stem()
        .unwrap()
        .to_string_lossy();

    prep_frame_tmp_dir(&frame_tmp_dir)?;

    let on_color = image::Rgb(colors::GREEN);
    let off_color = image::Rgb(colors::RED);
    let black = image::Rgb(colors::BLACK);

    const BUF_SIZE: usize = 150;
    let mut write_buf: Vec<Frame> = Vec::with_capacity(BUF_SIZE);

    // Init canvas
    let mut img = ImageBuffer::from_fn(u32::from(cam.camera_x), u32::from(cam.camera_y), |_, _| {
        image::Rgb(colors::BLACK)
    });

    let mut events_in_current_frame = 0;
    let max_events = config.window_size;
    let mut frames_created = 0;

    for event in events {
        // Place a pixel on the image canvas with the appropriate color & position
        place_pixel(config, cam, on_color, off_color, &mut img, &event);
        events_in_current_frame += 1;

        if events_in_current_frame == max_events {
            frames_created += 1;
            events_in_current_frame = 0;

            if frames_created == config.max_frames {
                break;
            }

            let count = fs::read_dir(&frame_tmp_dir)?.count() + write_buf.len();

            // Save image to buffer
            write_buf.push(Frame {
                img: img.clone(),
                count,
            });

            // Write all frames to disk once enough have been saved to buffer
            if write_buf.len() == BUF_SIZE {
                for frame in &write_buf {
                    frame.save_frame(&frame_tmp_dir, &video_name)?;
                }
                write_buf.clear();
            }

            // Reset canvas to black
            for pixel in img.pixels_mut() {
                *pixel = black;
            }
        }
    }

    // Save any remaining frames in buffer
    for frame in &write_buf {
        frame.save_frame(&frame_tmp_dir, &video_name)?;
    }

    // Save any remaining events in current working img
    let count = std::fs::read_dir(&frame_tmp_dir)?.count();
    img.save(format!(
        "{}/{}_tmp_{}.png",
        frame_tmp_dir.display(),
        video_name,
        count
    ))
    .unwrap();

    if !config.omit_video {
        encode_frames(&config.filename.to_string_lossy(), &frame_tmp_dir, cam)?;
    }

    if !config.keep_frames {
        prep_frame_tmp_dir(&frame_tmp_dir)?;
    }

    Ok(())
}

fn place_pixel(
    config: &VidConfig,
    cam: &CameraParameters,
    on_color: image::Rgb<u8>,
    off_color: image::Rgb<u8>,
    img: &mut ImageBuffer<image::Rgb<u8>, Vec<u8>>,
    event: &Event,
) {
    let (x, y) = event.get_coords(&cam.camera_type);

    let event_polarity = event.get_polarity(&cam.camera_type);

    if !config.exclude_on && event_polarity {
        img.put_pixel(u32::from(x - 1), u32::from(y - 1), on_color);
    } else if !config.exclude_off && !event_polarity {
        img.put_pixel(u32::from(x - 1), u32::from(y - 1), off_color);
    }
}

fn encode_frames(
    filename: &str,
    frame_tmp_dir: &PathBuf,
    cam: &CameraParameters,
) -> std::io::Result<()> {
    let paths = fs::read_dir(frame_tmp_dir).expect("Could not open frame tmp directory");
    let mut image_files: Vec<String> = paths
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path().display().to_string())
        .collect();

    image_files.sort_by(|a, b| natord::compare(a, b));

    // Set video properties
    let fourcc = VideoWriter::fourcc('M', 'J', 'P', 'G').unwrap();
    let fps = 30.0;
    let video_filename = filename.to_owned() + ".avi";

    // Create a VideoWriter
    let mut video = VideoWriter::new(
        &video_filename,
        fourcc,
        fps,
        Size::new(i32::from(cam.camera_x), i32::from(cam.camera_y)),
        true,
    )
    .expect("Could not initialize video writer");

    // Write each image frame to the video
    for image_file in image_files {
        let frame = imread(&image_file, IMREAD_COLOR).unwrap();
        video.write(&frame).expect("Could not encode file to video");
    }

    // Release the VideoWriter
    video.release().expect("Could not finalize video");

    Ok(())
}
