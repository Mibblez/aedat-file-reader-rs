pub mod aedat_utilities {
    use std::io::prelude::*;
    use std::fs::File;
    use std::convert::TryInto;
    use std::io::ErrorKind;

    pub struct Event {
        pub bytes: [u8; 8],
    }

    impl Event {
        pub fn get_polarity(&self) -> bool {
            //Event polarity is located in the fourth bit of the third byte
            (self.bytes[2] >> 3 & 1) == 1
        }

        pub fn get_timestamp(&self) -> i32 {
            // Timestamp is found in the last four bytes
            (((self.bytes[7] as u32) << 0) +
                ((self.bytes[6] as u32) << 8) +
                ((self.bytes[5] as u32) << 16) +
                ((self.bytes[4] as u32) << 24)) as i32
        }

        pub fn get_coords_dvs128(&self) -> (u8, u8) {
            // DVS128   (X = width - bits33-39 ) ; (Y = height - bits40-46 ) [bytes 2-3]
            (128 - ((self.bytes[3] >> 1) & 0b1111111) as u8, // X coordinate
             128 - (self.bytes[0] & 0b1111111) as u8)        // Y coordinate
        }

        // finish method pls. Same as 128 currently
        pub fn get_coords_davis240(&self) -> (u8, u8) {
            // DAVIS240  (X = width - bits51-44) ; (Y = height - bits60-54) [bytes 0-2]
            (240 - (((self.bytes[1] << 4) & 0b11110000) + ((self.bytes[2] >> 4) & 0b1111)) as u8,// X coordinate
             180 - (((self.bytes[0] << 2) & 0b01111100) + ((self.bytes[1] >> 6) & 0b11)) as u8)  // Y coordinate
        }
    }

    pub struct Config {
        pub filename: String,
        pub include_polarity: bool,
        pub coords: CoordMode,
    }

    impl Config {
        pub fn new(args: &[String]) -> Result<Config, std::io::Error> {
            if args.len() < 3 {
                return Err(std::io::Error::new(ErrorKind::InvalidInput,
                                               "Not enough arguments"));
            }

            let filename = args[1].clone();
            let include_polarity = match args[2].as_ref() {
                "-p" => true,
                "-np" => false,
                _ => return Err(std::io::Error::new(ErrorKind::InvalidInput,
                                                    "Invalid input"))
            };

            let coords = match args[3].as_ref() {
                "-nc" => CoordMode::NoCoord,
                "-xy" => CoordMode::XY,
                "-pn" => CoordMode::PixelNum,
                _ => return Err(std::io::Error::new(ErrorKind::InvalidInput,
                                                    "Invalid input"))
            };

            Ok(Config { filename, include_polarity, coords })
        }
    }

    pub enum CoordMode {
        NoCoord,
        XY,
        PixelNum,
    }

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
        pub fn new(camera_type: CameraType) -> CameraParameters {
            match camera_type {
                CameraType::DVS128 => CameraParameters{camera_type, camera_x: 128, camera_y: 128},
                CameraType::DAVIS240 => CameraParameters{camera_type, camera_x: 240, camera_y: 180},
            }
        }
    }

    pub fn find_line_in_header(aedat_file: &Vec<u8>, search: &str) -> Result<String, std::io::Error> {
        // Grab 0.5MB or the entire file if too small
        let header = match aedat_file {
            file if file.len() >= 524288 => &aedat_file[0..524288],
            _ => &aedat_file,
        };

        let contents = String::from_utf8_lossy(header);

        for line in contents.lines() {
            if line.contains(search) { return Ok(String::from(line))}
        }

        return Err(std::io::Error::new(
            ErrorKind::NotFound, format!("'{}' was not found in the file", search)));


    }

    pub fn parse_camera_type(aedat_file: &Vec<u8>) -> Result<CameraParameters, std::io::Error> {
        let hardware_interface = find_line_in_header(&aedat_file, "# HardwareInterface:")?;

        match Some(hardware_interface) {
            Some(ref s) if (s.contains("DVS128")) =>
                return Ok(CameraParameters::new(CameraType::DVS128)),
            Some(ref s) if (s.contains("DAVIS240")) =>
                return Ok(CameraParameters::new(CameraType::DAVIS240)),
            _ => return Err(std::io::Error::new(ErrorKind::NotFound, "Could not parse camera type")),
        };
    }

    pub fn find_header_end(aedat_file: &Vec<u8>) -> Result<u32, std::io::Error> {
        // Equivalent to: #End Of ASCII
        const END_OF_ASCII: [u8; 22] = [35, 69, 110, 100, 32, 79, 102, 32, 65, 83, 67, 73, 73, 32, 72, 101, 97, 100, 101, 114, 13, 10];

        let mut header_end_q = Vec::with_capacity(END_OF_ASCII.len());

        for (i, &item) in aedat_file.iter().enumerate() {

            header_end_q.push(item);

            // Pop oldest value off the queue if it becomes too large
            if header_end_q.len() > END_OF_ASCII.len() {
                header_end_q.remove(0);
            }

            // End of header has been found
            if &END_OF_ASCII[..] == &header_end_q[..] { return Ok((i + 1) as u32); }
        }

        return Err(std::io::Error::new(ErrorKind::NotFound, "End of header not found"));
    }


    pub fn get_events(end_of_header_index: u32, aedat_file: &Vec<u8>) -> Result<Vec<Event>, std::io::Error> {
        // Size of an event in bytes
        const EVENT_SIZE: usize = 8;

        // Skip over the header to get directly to the event data
        let aedat_iter = aedat_file.iter().skip(end_of_header_index as usize).enumerate();

        // Pre-allocate space in vec for all events
        let mut events = Vec::with_capacity(aedat_iter.len() / EVENT_SIZE);
        let mut bytes_tmp = Vec::with_capacity(EVENT_SIZE);

        // Loop over all bytes in file
        for (_i, &item) in aedat_iter {
            bytes_tmp.push(item.to_owned());

            // Collected enough bytes for an event. Create event and push to Vec of events
            if bytes_tmp.len() == EVENT_SIZE {
                let event = Event {
                    bytes: bytes_tmp[..].try_into().
                        expect("Slice with incorrect length")
                };
                bytes_tmp.clear();
                events.push(event);
            }
        }
        Ok(events)
    }

    fn format_polarity(polarity: bool) -> String {
        format!("{},", match polarity {
            true => "1",
            false => "-1"
        })
    }

    fn config_header(config: &Config) -> String {
        let mut header_tmp = String::from("");

        if config.include_polarity == true {
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
        format!("{x},{y},", x = x, y = y)
    }

    fn format_coords_pn(x: u8, y: u8, cam_x: &u8) -> String {
        format!("{},", ((*cam_x as u32 * (y - 1) as u32) + (x - 1) as u32))
    }

    pub fn create_csv(events: Vec<Event>, filename: &str, config: &Config, cam: &CameraParameters) -> std::io::Result<()> {
        // Create CSV file and write header
        let mut new_csv = File::create(filename)?;
        let csv_header = config_header(&config);
        new_csv.write(csv_header.as_bytes())?;

        // Create write buffer and preallocate space
        const BUF_PREALLOCATE_SIZE: usize = 150000;
        let mut write_buf = Vec::with_capacity(BUF_PREALLOCATE_SIZE);

        for event in events {
            let (x, y) = match cam.camera_type {
                CameraType::DVS128 => event.get_coords_dvs128(),
                CameraType::DAVIS240 => event.get_coords_davis240(),
            };

            write!(&mut write_buf, "{}",
                   format!("{p}{xy}{t}\n",
                           p = match config.include_polarity {
                               true => format_polarity(event.get_polarity()),
                               false => String::from(""),
                           },
                           xy = match config.coords {
                               CoordMode:: XY => format_coords_xy(x, y),
                               CoordMode:: PixelNum => format_coords_pn(x, y, &cam.camera_x),
                               CoordMode:: NoCoord => String::from(""),

                           },
                           t = event.get_timestamp()))?;

            // Write events to disk once enough have been collected
            if write_buf.len() >= BUF_PREALLOCATE_SIZE {
                new_csv.write(write_buf.as_slice())?;
                write_buf.clear();
            }
        }

        // Write any remaining events to disk
        if !write_buf.is_empty() {
            new_csv.write(write_buf.as_slice())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::aedat_utilities::*;

    #[test]
    fn event_test_dvs128() {
        let test_event_bytes: [u8; 8] = [0, 0, 56, 231, 156, 86, 232, 205];
        let test_event_struct = Event { bytes: test_event_bytes };

        // Get event polarity
        let polarity = test_event_struct.get_polarity();
        assert_eq!(polarity, true);

        // Get timestamp
        let timestamp = test_event_struct.get_timestamp();
        assert_eq!(timestamp, -1672025907);

        // Get XY coordinates
        let (x, y) = test_event_struct.get_coords_dvs128();
        assert_eq!(x, 13);
        assert_eq!(y, 72);
    }

    fn read_test_file(file_path: &str) -> Vec<u8> {
        use std::io::prelude::*;
        use std::fs::File;

        // Read file
        let mut f = File::open(file_path)
            .expect("Could not access test file");
        let mut aedat_file = Vec::new();
        f.read_to_end(&mut aedat_file).expect("Could not read test file");

        aedat_file
    }

    #[test]
    fn header_end_test_128() {
        let aedat_file = read_test_file("test_files/header_test_128.txt");

        let header_end =  match find_header_end(&aedat_file) {
            Ok(t) => t,
            Err(e) => panic!("Could not find end of header {}", e),
        };

        assert_eq!(header_end, 4241);
    }

    #[test]
    fn header_end_test_240() {
        let aedat_file = read_test_file("test_files/header_test_240.txt");

        let header_end =  match find_header_end(&aedat_file) {
            Ok(t) => t,
            Err(e) => panic!("Could not find end of header {}", e),
        };

        assert_eq!(header_end, 303869);
    }

    #[test]
    fn camera_type_test_128() {
        let aedat_file = read_test_file("test_files/header_test_128.txt");

        let cam = match parse_camera_type(&aedat_file) {
            Ok(t) => t,
            Err(e) => panic!(e),
        };

        assert_eq!(cam.camera_x, 128);
        assert_eq!(cam.camera_y, 128);

    }

    #[test]
    fn camera_type_test_240() {
        let aedat_file = read_test_file("test_files/header_test_240.txt");

        let cam = match parse_camera_type(&aedat_file) {
            Ok(t) => t,
            Err(e) => panic!(e),
        };

        assert_eq!(cam.camera_x, 240);
        assert_eq!(cam.camera_y, 180);

    }
}