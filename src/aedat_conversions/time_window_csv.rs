use std::{
    fs::File,
    io::{self, Write},
};

use crate::{
    aedat_data::{CameraParameters, Event},
    cli_configs::TimeWindowConfig,
};

pub struct Downres {
    pixels: Vec<usize>,
    size_x: usize,
    size_y: usize,
    size_x_downscaled: usize,
    size_y_downscaled: usize,
    scale: usize,
    threshold: usize,
}

impl Downres {
    pub fn new(size_x: usize, size_y: usize, scale: usize, threshold: usize) -> Self {
        Downres {
            pixels: vec![0; size_x / scale * size_y / scale],
            size_x,
            size_y,
            size_x_downscaled: size_x / scale,
            size_y_downscaled: size_y / scale,
            scale,
            threshold,
        }
    }

    pub fn reset(&mut self) {
        self.pixels = vec![0; self.size_x_downscaled * self.size_y_downscaled];
    }

    #[allow(dead_code)]
    pub fn get_pixel(&self, x: usize, y: usize) -> Option<usize> {
        if x > 0 && x <= self.size_x && y > 0 && y <= self.size_y {
            Some(
                self.pixels
                    [(((y - 1) / self.scale) * self.size_x_downscaled) + ((x - 1) / self.scale)],
            )
        } else {
            None
        }
    }

    pub fn increment_pixel(&mut self, x: usize, y: usize) -> Result<(), std::io::Error> {
        if x > 0 && x <= self.size_x && y > 0 && y <= self.size_y {
            self.pixels
                [(((y - 1) / self.scale) * self.size_x_downscaled) + ((x - 1) / self.scale)] += 1;
            Ok(())
        } else {
            // Err("Index out of bounds")
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Index out of bounds",
            ))
        }
    }

    pub fn to_pgm(&self) -> String {
        let mut result = String::from(&format!(
            "P2\n{} {}\n1\n",
            self.size_x_downscaled, self.size_y_downscaled
        ));

        let pgm_width = self.size_x_downscaled;

        for (i, val) in self.pixels.iter().enumerate() {
            result.push_str(&format!(
                "{}{}",
                i32::from(val >= &self.threshold),
                if i % pgm_width == pgm_width - 1 {
                    "\n"
                } else {
                    " "
                }
            ));
        }

        result
    }
}

pub fn create_time_window_csv(
    events: Vec<Event>,
    config: &TimeWindowConfig,
    cam: &CameraParameters,
) -> std::io::Result<()> {
    // Create CSV file and write header
    let mut new_csv = File::create(&config.filename)?;
    let csv_header = config.create_csv_header();
    new_csv.write_all(csv_header.as_bytes())?;

    // Create write buffer and preallocate space
    const BUF_SIZE: usize = 150_000;
    let mut write_buf = Vec::with_capacity(BUF_SIZE);

    let mut end_time = match events.first() {
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "No events exist",
            ))
        }
        Some(event) => event.get_timestamp() + config.window_size as i32,
    };

    let mut on_count = 0;
    let mut off_count = 0;
    let mut windows_created = 0;

    // Downscaled PGM image
    let mut downres = Downres::new(
        cam.camera_x as usize,
        cam.camera_y as usize,
        config.pgm_scale,
        config.pgm_threshold,
    );

    for event in events {
        if event.get_timestamp() > end_time {
            writeln!(
                &mut write_buf,
                "{on},{off}{both}{downres_pgm}",
                on = on_count,
                off = off_count,
                both = if config.include_both_column {
                    format!(",{}", on_count + off_count)
                } else {
                    String::new()
                },
                downres_pgm = if config.include_pgm {
                    format!(",{}", downres.to_pgm().replace('\n', "-"))
                } else {
                    String::new()
                }
            )?;

            downres.reset();

            // Write events to disk once enough have been collected
            if write_buf.len() >= BUF_SIZE {
                new_csv.write_all(write_buf.as_slice())?;
                write_buf.clear();
            }

            end_time += config.window_size as i32;
            on_count = 0;
            off_count = 0;

            windows_created += 1;
            if windows_created == config.max_windows {
                break;
            }
        }

        if event.get_polarity(&cam.camera_type) {
            on_count += 1;
        } else {
            off_count += 1;
        }

        if config.include_pgm {
            let (x, y) = event.get_coords(&cam.camera_type);
            downres.increment_pixel(x as usize, y as usize)?;
        }
    }

    // Write any remaining events to disk
    if !write_buf.is_empty() {
        new_csv.write_all(write_buf.as_slice())?;
    }

    Ok(())
}
