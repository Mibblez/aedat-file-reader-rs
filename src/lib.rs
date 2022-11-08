pub mod aedat_utilities {
    extern crate clap;
    extern crate image;

    use std::convert::TryInto;
    use std::fs;
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::Error;
    use std::io::ErrorKind;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use image::ImageBuffer;
    use image::Rgb;

    use clap::ArgMatches;

    mod colors {
        pub static RED: [u8; 3] = [255u8, 0u8, 0u8];
        pub static GREEN: [u8; 3] = [0u8, 255u8, 0u8];
        pub static BLACK: [u8; 3] = [0u8, 0u8, 0u8];
    }

    pub struct Event {
        pub bytes: [u8; 8],
    }

    impl Event {
        pub fn get_polarity(&self, cam_type: &CameraType) -> bool {
            match cam_type {
                CameraType::DVS128 => (self.bytes[3] & 1) == 1, // first bit of the fourth byte
                CameraType::DAVIS240 => ((self.bytes[2] >> 3) & 1) == 1, // fourth bit of the third byte
            }
        }

        pub fn get_timestamp(&self) -> i32 {
            // Timestamp is found in the last four bytes
            (((self.bytes[7] as u32) << 0)
                + ((self.bytes[6] as u32) << 8)
                + ((self.bytes[5] as u32) << 16)
                + ((self.bytes[4] as u32) << 24)) as i32
        }

        pub fn get_coords(&self, cam_type: &CameraType) -> (u8, u8) {
            match cam_type {
                CameraType::DVS128 => {
                    // DVS128   (X = width - bits33-39 ) ; (Y = height - bits40-46 ) [bytes 2-3]
                    (
                        128 - ((self.bytes[3] >> 1) & 0b1111111) as u8, // X coordinate
                        128 - (self.bytes[2] & 0b1111111) as u8,
                    ) // Y coordinate
                }
                CameraType::DAVIS240 => {
                    // DAVIS240  (X = width - bits51-44) ; (Y = height - bits60-54) [bytes 0-2]
                    (
                        240 - (((self.bytes[1] << 4) & 0b11110000)
                            + ((self.bytes[2] >> 4) & 0b1111)) as u8, // X coordinate
                        180 - (((self.bytes[0] << 2) & 0b01111100) + ((self.bytes[1] >> 6) & 0b11))
                            as u8,
                    ) // Y coordinate
                }
            }
        }
    }

    pub struct CsvConfig {
        pub filename: PathBuf,
        pub include_polarity: bool,
        pub coords: CoordMode,
        pub offset_time: bool,
    }

    impl CsvConfig {
        pub fn new(args: &ArgMatches) -> Result<CsvConfig, std::io::Error> {
            let mut filename = args.get_one::<PathBuf>("filename").unwrap().to_owned();
            filename.set_extension("csv");

            let include_polarity = args.get_flag("includePolarity");
            let exclude_polarity = args.get_flag("excludePolarity");

            let include_polarity = match (include_polarity, exclude_polarity) {
                (true, _) => true,
                (_, true) => false,
                _ => unreachable!(),
            };

            let coords = args.get_flag("coords");
            let pixel_number = args.get_flag("pixelNumber");
            let no_spatial = args.get_flag("noSpatial");

            let coords = match (coords, pixel_number, no_spatial) {
                (true, _, _) => CoordMode::XY,
                (_, true, _) => CoordMode::PixelNum,
                (_, _, true) => CoordMode::NoCoord,
                _ => unreachable!(),
            };

            let offset_time = args.get_flag("offsetTime");

            Ok(CsvConfig {
                filename,
                include_polarity,
                coords,
                offset_time,
            })
        }
    }

    pub struct VidConfig {
        pub filename: PathBuf,
        pub window_size: usize,
        pub max_frames: usize,
        pub exclude_on: bool,
        pub exclude_off: bool,
        pub keep_frames: bool,
        pub omit_video: bool,
    }

    impl VidConfig {
        pub fn new(args: &ArgMatches) -> Result<VidConfig, std::io::Error> {
            let mut filename = args.get_one::<PathBuf>("filename").unwrap().to_owned();
            filename.set_extension("");

            let window_size: usize = args.get_one::<usize>("windowSize").unwrap().to_owned();

            let max_frames: usize = match args.get_one::<usize>("maxFrames") {
                Some(v) => v.to_owned(),
                None => std::usize::MAX,
            };

            let exclude_on = args.get_flag("excludeOnEvents");
            let exclude_off = args.get_flag("excludeOffEvents");

            let keep_frames = args.get_flag("keepFrames");
            let omit_video = args.get_flag("omitVideo");

            Ok(VidConfig {
                filename,
                window_size,
                max_frames,
                exclude_on,
                exclude_off,
                keep_frames,
                omit_video,
            })
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

    pub struct Frame {
        pub img: ImageBuffer<Rgb<u8>, Vec<u8>>,
        pub count: usize,
    }

    impl Frame {
        pub fn save_frame(&self, frame_tmp_dir: &PathBuf, filename: &str) -> std::io::Result<()> {
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

    pub fn find_line_in_header(
        aedat_file: &Vec<u8>,
        search: &str,
    ) -> Result<String, std::io::Error> {
        // Grab 0.5MB or the entire file if too small
        let header = match aedat_file {
            file if file.len() >= 524288 => &aedat_file[0..524288],
            _ => &aedat_file,
        };

        let contents = String::from_utf8_lossy(header);

        for line in contents.lines() {
            if line.contains(search) {
                return Ok(String::from(line));
            }
        }

        return Err(std::io::Error::new(
            ErrorKind::NotFound,
            format!("'{}' was not found in the file", search),
        ));
    }

    pub fn parse_camera_type(aedat_file: &Vec<u8>) -> Result<CameraParameters, std::io::Error> {
        let hardware_interface = find_line_in_header(&aedat_file, "# HardwareInterface:")?;

        match Some(hardware_interface) {
            Some(ref s) if (s.contains("DVS128")) => {
                return Ok(CameraParameters::new(CameraType::DVS128))
            }
            Some(ref s) if (s.contains("DAVIS240")) => {
                return Ok(CameraParameters::new(CameraType::DAVIS240))
            }
            _ => {
                return Err(std::io::Error::new(
                    ErrorKind::NotFound,
                    "Could not parse camera type",
                ))
            }
        };
    }

    pub fn find_header_end(aedat_file: &Vec<u8>) -> Result<u32, std::io::Error> {
        // Equivalent to: #End Of ASCII
        const END_OF_ASCII: [u8; 22] = [
            35, 69, 110, 100, 32, 79, 102, 32, 65, 83, 67, 73, 73, 32, 72, 101, 97, 100, 101, 114,
            13, 10,
        ];

        let mut header_end_q = Vec::with_capacity(END_OF_ASCII.len());

        for (i, &item) in aedat_file.iter().enumerate() {
            header_end_q.push(item);

            // Pop oldest value off the queue if it becomes too large
            if header_end_q.len() > END_OF_ASCII.len() {
                header_end_q.remove(0);
            }

            // End of header has been found
            if &END_OF_ASCII[..] == &header_end_q[..] {
                return Ok((i + 1) as u32);
            }
        }

        return Err(std::io::Error::new(
            ErrorKind::NotFound,
            "End of header not found",
        ));
    }

    pub fn get_events(
        end_of_header_index: u32,
        aedat_file: &Vec<u8>,
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

    fn format_polarity(polarity: bool) -> String {
        format!(
            "{},",
            match polarity {
                true => "1",
                false => "-1",
            }
        )
    }

    fn config_header(config: &CsvConfig) -> String {
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

    pub fn create_csv(
        events: Vec<Event>,
        config: &CsvConfig,
        cam: &CameraParameters,
    ) -> std::io::Result<()> {
        // Create CSV file and write header
        let mut new_csv = File::create(&config.filename)?;
        let csv_header = config_header(&config);
        new_csv.write(csv_header.as_bytes())?;

        // Create write buffer and preallocate space
        const BUF_SIZE: usize = 150000;
        let mut write_buf = Vec::with_capacity(BUF_SIZE);

        let time_offset = match config.offset_time {
            true => events[0].get_timestamp(),
            false => 0,
        };

        for event in events {
            let (x, y) = event.get_coords(&cam.camera_type);
            let event_polarity = event.get_polarity(&cam.camera_type);

            write!(
                &mut write_buf,
                "{}",
                format!(
                    "{p}{xy}{t}\n",
                    p = match config.include_polarity {
                        true => format_polarity(event_polarity),
                        false => String::from(""),
                    },
                    xy = match config.coords {
                        CoordMode::XY => format_coords_xy(x, y),
                        CoordMode::PixelNum => format_coords_pn(x, y, &cam.camera_x),
                        CoordMode::NoCoord => String::from(""),
                    },
                    t = event.get_timestamp() - time_offset,
                )
            )?;

            // Write events to disk once enough have been collected
            if write_buf.len() >= BUF_SIZE {
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

    fn prep_frame_tmp_dir(tmp_dir: &PathBuf) -> std::io::Result<()> {
        // Create frame tmp directory if it does not exist
        match fs::create_dir(tmp_dir) {
            Ok(_) => (),
            Err(_) => (),
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
            config.filename.to_owned()
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
        let mut img = ImageBuffer::from_fn(cam.camera_x as u32, cam.camera_y as u32, |_, _| {
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
            place_pixel(config, cam, &on_color, &off_color, &mut img, &event);

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
            encode_frames(&config.filename.to_string_lossy(), &frame_tmp_dir)?;
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
            config.filename.to_owned()
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
        let mut img = ImageBuffer::from_fn(cam.camera_x as u32, cam.camera_y as u32, |_, _| {
            image::Rgb(colors::BLACK)
        });

        let mut events_in_current_frame = 0;
        let max_events = config.window_size;
        let mut frames_created = 0;

        for event in events {
            // Place a pixel on the image canvas with the appropriate color & position
            place_pixel(config, cam, &on_color, &off_color, &mut img, &event);
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
            encode_frames(&config.filename.to_string_lossy(), &frame_tmp_dir)?;
        }

        if !config.keep_frames {
            prep_frame_tmp_dir(&frame_tmp_dir)?;
        }

        Ok(())
    }

    fn place_pixel(
        config: &VidConfig,
        cam: &CameraParameters,
        on_color: &image::Rgb<u8>,
        off_color: &image::Rgb<u8>,
        img: &mut ImageBuffer<image::Rgb<u8>, Vec<u8>>,
        event: &Event,
    ) {
        let (x, y) = event.get_coords(&cam.camera_type);

        let event_polarity = event.get_polarity(&cam.camera_type);

        if !config.exclude_on && event_polarity == true {
            img.put_pixel((x - 1) as u32, (y - 1) as u32, *on_color);
        } else if !config.exclude_off && event_polarity == false {
            img.put_pixel((x - 1) as u32, (y - 1) as u32, *off_color);
        }
    }

    fn encode_frames(filename: &str, frame_tmp_dir: &PathBuf) -> std::io::Result<()> {
        // Encode images into a video via python script
        let output = Command::new("python3")
            .arg("src/frames_to_vid.py")
            .arg(format!("{}.avi", filename))
            .arg(frame_tmp_dir)
            .output()
            .expect("failed to execute process");

        let python_msg = String::from_utf8_lossy(&output.stdout).to_string();

        // Check for errors in python script
        if &python_msg != "0" {
            // Clear tmp files
            let paths = fs::read_dir(frame_tmp_dir)?;
            for path in paths {
                fs::remove_file(path?.path())?;
            }

            return match python_msg.as_ref() {
                "1" => Err(std::io::Error::new(
                    ErrorKind::Other,
                    "Unmet Python dependency in frames_to_vid.py",
                )),
                "2" => Err(std::io::Error::new(
                    ErrorKind::Other,
                    "frames_to_vid.py must be run with Python3",
                )),
                _ => Err(std::io::Error::new(
                    ErrorKind::Other,
                    "Unknown error in frames_to_vid.py",
                )),
            };
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
        let test_event_struct = Event {
            bytes: test_event_bytes,
        };

        // Get event polarity
        let polarity = test_event_struct.get_polarity(&CameraType::DVS128);
        assert_eq!(polarity, true);

        // Get timestamp
        let timestamp = test_event_struct.get_timestamp();
        assert_eq!(timestamp, -1672025907);

        // Get XY coordinates
        let (x, y) = test_event_struct.get_coords(&CameraType::DVS128);
        assert_eq!(x, 13);
        assert_eq!(y, 72);
    }

    fn read_test_file(file_path: &str) -> Vec<u8> {
        use std::fs::File;
        use std::io::prelude::*;

        // Read file
        let mut f = File::open(file_path).expect("Could not access test file");
        let mut aedat_file = Vec::new();
        f.read_to_end(&mut aedat_file)
            .expect("Could not read test file");

        aedat_file
    }

    #[test]
    fn header_end_test_128() {
        let aedat_file = read_test_file("test_files/header_test_128.aedat_test");

        // println!("{:?}", aedat_file);

        let header_end = match find_header_end(&aedat_file) {
            Ok(t) => t,
            Err(e) => panic!("{}", e),
        };

        assert_eq!(header_end, 4241);
    }

    #[test]
    fn header_end_test_240() {
        let aedat_file = read_test_file("test_files/header_test_240.aedat_test");

        let header_end = match find_header_end(&aedat_file) {
            Ok(t) => t,
            Err(e) => panic!("{}", e),
        };

        assert_eq!(header_end, 303869);
    }

    #[test]
    fn camera_type_test_128() {
        let aedat_file = read_test_file("test_files/header_test_128.aedat_test");

        let cam = match parse_camera_type(&aedat_file) {
            Ok(t) => t,
            Err(e) => panic!("{}", e),
        };

        assert_eq!(cam.camera_x, 128);
        assert_eq!(cam.camera_y, 128);
    }

    #[test]
    fn camera_type_test_240() {
        let aedat_file = read_test_file("test_files/header_test_240.aedat_test");

        let cam = match parse_camera_type(&aedat_file) {
            Ok(t) => t,
            Err(e) => panic!("{}", e),
        };

        assert_eq!(cam.camera_x, 240);
        assert_eq!(cam.camera_y, 180);
    }
}
