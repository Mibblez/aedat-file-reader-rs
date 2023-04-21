#[cfg(test)]
mod tests {
    use crate::{
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
}
