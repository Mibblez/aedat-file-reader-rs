use std::{fs::File, io::Write};

use crate::{
    aedat_data::{CameraParameters, Event},
    cli_configs::{CoordMode, CsvConfig},
};

fn format_polarity(polarity: bool) -> String {
    format!("{},", if polarity { "1" } else { "-1" })
}

fn config_csv_header(config: &CsvConfig) -> String {
    let mut header_tmp = String::new();

    if config.include_polarity {
        header_tmp.push_str("On/Off,");
    }

    match config.coords {
        CoordMode::NoCoord => (),
        CoordMode::XY => header_tmp.push_str("X,Y,"),
        CoordMode::PixelNum => header_tmp.push_str("Pixel Number,"),
    };

    header_tmp.push_str("Timestamp\n");

    header_tmp
}

fn format_coords_xy(x: u8, y: u8) -> String {
    format!("{x},{y},")
}

fn format_coords_pn(x: u8, y: u8, cam_x: u8) -> String {
    format!(
        "{},",
        ((u32::from(cam_x) * u32::from(y - 1)) + u32::from(x - 1))
    )
}

pub fn create_csv(
    events: Vec<Event>,
    config: &CsvConfig,
    cam: &CameraParameters,
) -> std::io::Result<()> {
    // Create CSV file and write header
    let mut new_csv = File::create(&config.filename)?;
    let csv_header = config_csv_header(config);
    new_csv.write_all(csv_header.as_bytes())?;

    // Create write buffer and preallocate space
    const BUF_SIZE: usize = 150_000;
    let mut write_buf = Vec::with_capacity(BUF_SIZE);

    let time_offset = if config.offset_time {
        events[0].get_timestamp()
    } else {
        0
    };

    for event in events {
        let (x, y) = event.get_coords(&cam.camera_type);
        let event_polarity = event.get_polarity(&cam.camera_type);

        writeln!(
            &mut write_buf,
            "{p}{xy}{t}",
            p = if config.include_polarity {
                format_polarity(event_polarity)
            } else {
                String::new()
            },
            xy = match config.coords {
                CoordMode::XY => format_coords_xy(x, y),
                CoordMode::PixelNum => format_coords_pn(x, y, cam.camera_x),
                CoordMode::NoCoord => String::new(),
            },
            t = event.get_timestamp() - time_offset,
        )?;

        // Write events to disk once enough have been collected
        if write_buf.len() >= BUF_SIZE {
            new_csv.write_all(write_buf.as_slice())?;
            write_buf.clear();
        }
    }

    // Write any remaining events to disk
    if !write_buf.is_empty() {
        new_csv.write_all(write_buf.as_slice())?;
    }

    Ok(())
}
