use std::{fs::File, io::Write};

use crate::{
    aedat_data::{CameraParameters, Event},
    cli_configs::TimeWindowConfig,
};

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

    for event in events {
        if event.get_timestamp() > end_time {
            writeln!(
                &mut write_buf,
                "{on},{off}{both}",
                on = on_count,
                off = off_count,
                both = if config.include_both_column {
                    format!(",{}", on_count + off_count)
                } else {
                    String::new()
                }
            )?;

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
    }

    // Write any remaining events to disk
    if !write_buf.is_empty() {
        new_csv.write_all(write_buf.as_slice())?;
    }

    Ok(())
}
