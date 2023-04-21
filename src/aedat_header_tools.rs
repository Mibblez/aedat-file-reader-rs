use std::io::ErrorKind;

use crate::aedat_data::{CameraParameters, CameraType};

fn find_line_in_header(aedat_file: &Vec<u8>, search: &str) -> Result<String, std::io::Error> {
    // Grab 0.5MB or the entire file if too small
    let header = match aedat_file {
        file if file.len() >= 524_288 => &aedat_file[0..524_288],
        _ => aedat_file,
    };

    let contents = String::from_utf8_lossy(header);

    for line in contents.lines() {
        if line.contains(search) {
            return Ok(String::from(line));
        }
    }

    Err(std::io::Error::new(
        ErrorKind::NotFound,
        format!("'{search}' was not found in the file"),
    ))
}

pub fn parse_camera_type(aedat_file: &Vec<u8>) -> Result<CameraParameters, std::io::Error> {
    let hardware_interface = find_line_in_header(aedat_file, "# HardwareInterface:")?;

    match Some(hardware_interface) {
        Some(ref s) if (s.contains("DVS128")) => Ok(CameraParameters::new(CameraType::DVS128)),
        Some(ref s) if (s.contains("DAVIS240")) => Ok(CameraParameters::new(CameraType::DAVIS240)),
        _ => Err(std::io::Error::new(
            ErrorKind::NotFound,
            "Could not parse camera type",
        )),
    }
}

pub fn find_header_end(aedat_file: &[u8]) -> Result<u32, std::io::Error> {
    // Equivalent to: #End Of ASCII
    const END_OF_ASCII: [u8; 22] = [
        35, 69, 110, 100, 32, 79, 102, 32, 65, 83, 67, 73, 73, 32, 72, 101, 97, 100, 101, 114, 13,
        10,
    ];

    let mut header_end_q = Vec::with_capacity(END_OF_ASCII.len());

    for (i, &item) in aedat_file.iter().enumerate() {
        header_end_q.push(item);

        // Pop oldest value off the queue if it becomes too large
        if header_end_q.len() > END_OF_ASCII.len() {
            header_end_q.remove(0);
        }

        // End of header has been found
        if END_OF_ASCII[..] == header_end_q[..] {
            return Ok((i + 1) as u32);
        }
    }

    Err(std::io::Error::new(
        ErrorKind::NotFound,
        "End of header not found",
    ))
}
