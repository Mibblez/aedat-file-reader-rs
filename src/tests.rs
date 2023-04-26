#[cfg(test)]
mod tests {
    use crate::{
        aedat_conversions::time_window_csv::Downres,
        aedat_data::{CameraType, Event},
        aedat_header_tools::{find_header_end, parse_camera_type},
    };

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

    #[test]
    fn downres_bounds() {
        let downres_128 = Downres::new(128, 128, 4, 4);
        assert_eq!(downres_128.get_pixel(128, 128), Some(0));
        assert_eq!(downres_128.get_pixel(129, 129), None);
        assert_eq!(downres_128.get_pixel(1, 1), Some(0));
        assert_eq!(downres_128.get_pixel(0, 0), None);

        let downres_240 = Downres::new(240, 180, 4, 4);
        assert_eq!(downres_240.get_pixel(240, 180), Some(0));
        assert_eq!(downres_240.get_pixel(241, 181), None);
        assert_eq!(downres_240.get_pixel(1, 1), Some(0));
        assert_eq!(downres_240.get_pixel(0, 0), None);
    }

    #[test]
    fn downres_scale() {
        let mut downres_128 = Downres::new(128, 128, 4, 4);

        for i in 13..=16 {
            for j in 13..=16 {
                if let Err(e) = downres_128.increment_pixel(i, j) {
                    panic!("{}", e)
                };
            }
        }

        for i in 13..=16 {
            for j in 13..=16 {
                assert_eq!(downres_128.get_pixel(i, j), Some(16));
            }
        }

        if let Err(e) = downres_128.increment_pixel(10, 10) {
            panic!("{}", e)
        };
        if let Err(e) = downres_128.increment_pixel(10, 10) {
            panic!("{}", e)
        };
        assert_eq!(downres_128.get_pixel(10, 10), Some(2));

        assert_eq!(downres_128.get_pixel(50, 50), Some(0));
    }

    #[test]
    fn downres_to_pgm() {
        let downres_128_scale4 = Downres::new(128, 128, 4, 1);
        let downres_128_scale4_pgm = downres_128_scale4.to_pgm();
        let downres_128_scale4_resolution =
            downres_128_scale4_pgm.split("\n").collect::<Vec<_>>()[1];
        assert_eq!(downres_128_scale4_resolution, "32 32");

        let mut downres_128_scale16 = Downres::new(128, 128, 16, 1);

        assert_eq!(
            downres_128_scale16.to_pgm(),
            "P2\n8 8\n1\n\
        0 0 0 0 0 0 0 0\n\
        0 0 0 0 0 0 0 0\n\
        0 0 0 0 0 0 0 0\n\
        0 0 0 0 0 0 0 0\n\
        0 0 0 0 0 0 0 0\n\
        0 0 0 0 0 0 0 0\n\
        0 0 0 0 0 0 0 0\n\
        0 0 0 0 0 0 0 0\n"
        );

        if let Err(e) = downres_128_scale16.increment_pixel(1, 1) {
            panic!("{}", e)
        };
        if let Err(e) = downres_128_scale16.increment_pixel(64, 64) {
            panic!("{}", e)
        };
        if let Err(e) = downres_128_scale16.increment_pixel(128, 128) {
            panic!("{}", e)
        };

        assert_eq!(
            downres_128_scale16.to_pgm(),
            "P2\n8 8\n1\n\
        1 0 0 0 0 0 0 0\n\
        0 0 0 0 0 0 0 0\n\
        0 0 0 0 0 0 0 0\n\
        0 0 0 1 0 0 0 0\n\
        0 0 0 0 0 0 0 0\n\
        0 0 0 0 0 0 0 0\n\
        0 0 0 0 0 0 0 0\n\
        0 0 0 0 0 0 0 1\n"
        );

        downres_128_scale16.reset();

        assert_eq!(
            downres_128_scale16.to_pgm(),
            "P2\n8 8\n1\n\
        0 0 0 0 0 0 0 0\n\
        0 0 0 0 0 0 0 0\n\
        0 0 0 0 0 0 0 0\n\
        0 0 0 0 0 0 0 0\n\
        0 0 0 0 0 0 0 0\n\
        0 0 0 0 0 0 0 0\n\
        0 0 0 0 0 0 0 0\n\
        0 0 0 0 0 0 0 0\n"
        );
    }
}
