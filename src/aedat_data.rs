#[derive(Debug)]
pub enum CameraType {
    DVS128,
    DAVIS240,
}

pub struct CameraParameters {
    pub camera_type: CameraType,
    pub camera_x: u8,
    pub camera_y: u8,
}

impl CameraParameters {
    #[must_use]
    pub fn new(camera_type: CameraType) -> CameraParameters {
        match camera_type {
            CameraType::DVS128 => CameraParameters {
                camera_type,
                camera_x: 128,
                camera_y: 128,
            },
            CameraType::DAVIS240 => CameraParameters {
                camera_type,
                camera_x: 240,
                camera_y: 180,
            },
        }
    }
}

pub struct Event {
    pub bytes: [u8; 8],
}

impl Event {
    #[must_use]
    pub fn get_polarity(&self, cam_type: &CameraType) -> bool {
        match cam_type {
            CameraType::DVS128 => (self.bytes[3] & 1) == 1, // first bit of the fourth byte
            CameraType::DAVIS240 => ((self.bytes[2] >> 3) & 1) == 1, // fourth bit of the third byte
        }
    }

    #[must_use]
    pub fn get_timestamp(&self) -> i32 {
        // Timestamp is found in the last four bytes
        (i32::from(self.bytes[7]))
            + ((i32::from(self.bytes[6])) << 8)
            + ((i32::from(self.bytes[5])) << 16)
            + ((i32::from(self.bytes[4])) << 24)
    }

    #[must_use]
    pub fn get_coords(&self, cam_type: &CameraType) -> (u8, u8) {
        match cam_type {
            CameraType::DVS128 => {
                // DVS128   (X = width - bits33-39 ) ; (Y = height - bits40-46 ) [bytes 2-3]
                (
                    // X coordinate
                    128 - ((self.bytes[3] >> 1) & 0b111_1111),
                    // Y coordinate
                    128 - (self.bytes[2] & 0b111_1111),
                )
            }
            CameraType::DAVIS240 => {
                // DAVIS240  (X = width - bits44-51) ; (Y = height - bits54-61) [bytes 0-2]
                (
                    // X coordinate
                    240 - (((self.bytes[1] << 4) & 0b1111_0000) + ((self.bytes[2] >> 4) & 0b1111)),
                    // Y coordinate
                    180 - (((self.bytes[0] << 2) & 0b1111_1100) + ((self.bytes[1] >> 6) & 0b11)),
                )
            }
        }
    }
}

pub fn get_events(
    end_of_header_index: u32,
    aedat_file: &[u8],
) -> Result<Vec<Event>, std::io::Error> {
    // Size of an event in bytes
    const EVENT_SIZE: usize = 8;

    // Skip over the header to get directly to the event data
    let aedat_iter = aedat_file
        .iter()
        .skip(end_of_header_index as usize)
        .enumerate();

    // Pre-allocate space in vec for all events
    let mut events = Vec::with_capacity(aedat_iter.len() / EVENT_SIZE);
    let mut bytes_tmp = Vec::with_capacity(EVENT_SIZE);

    // Loop over all bytes in file
    for (_i, &item) in aedat_iter {
        bytes_tmp.push(item.to_owned());

        // Collected enough bytes for an event. Create event and push to Vec of events
        if bytes_tmp.len() == EVENT_SIZE {
            let event = Event {
                bytes: bytes_tmp[..]
                    .try_into()
                    .expect("Slice with incorrect length"),
            };
            bytes_tmp.clear();
            events.push(event);
        }
    }
    Ok(events)
}
