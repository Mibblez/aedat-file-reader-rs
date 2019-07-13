mod lib;

pub use crate::lib::aedat_utilities;

use std::io::prelude::*;
use std::process;
use std::env;
use std::fs::File;
use std::path::Path;


fn main() {
    // Get environment variables
    let args: Vec<String> = env::args().collect();

    if &args[2] == "-csv" {
        let csv_config = aedat_utilities::CsvConfig::new(&args).unwrap_or_else(|err| {
            eprintln!("Problem parsing arguments: {}", err);
            process::exit(1);
        });

        // Read file
        let mut f = File::open(&csv_config.filename).unwrap();
        let mut aedat_file = Vec::new();
        f.read_to_end(&mut aedat_file).unwrap();

        let cam = aedat_utilities::parse_camera_type(&aedat_file).unwrap();
        // println!("Found camera: {:?}", cam.camera_type);

        let header_end = aedat_utilities::find_header_end(&aedat_file).unwrap();
        // println!("End of header at position: {:?}", header_end);

        let events = aedat_utilities::get_events(header_end, &aedat_file).unwrap();
        // println!("Total number of events: {}", events.len());

        use std::time::Instant;
        let now = Instant::now();

        // TODO: should probably fix this mess
        let csv_name = Path::new(&csv_config.filename).file_stem().unwrap().to_str().unwrap();
        aedat_utilities::create_csv(events, csv_name, &csv_config, &cam).unwrap();

        let elapsed = now.elapsed();
        let sec = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
        println!("Export time: {} seconds", sec);
    }
    else if &args[2] == "-vid" {
        let vid_config = aedat_utilities::VidConfig::new(&args).unwrap_or_else(|err| {
            eprintln!("Problem parsing arguments: {}", err);
            process::exit(1);
        });

        // Read file
        let mut f = File::open(&vid_config.filename).unwrap();
        let mut aedat_file = Vec::new();
        f.read_to_end(&mut aedat_file).unwrap();

        let cam = aedat_utilities::parse_camera_type(&aedat_file).unwrap();
        // println!("Found camera: {:?}", cam.camera_type);

        let header_end = aedat_utilities::find_header_end(&aedat_file).unwrap();
        // println!("End of header at position: {:?}", header_end);

        let events = aedat_utilities::get_events(header_end, &aedat_file).unwrap();
        // println!("Total number of events: {}", events.len());

        // TODO: should probably fix this mess
        let video_name = Path::new(&vid_config.filename).file_stem().unwrap().to_str().unwrap();

        aedat_utilities::create_video(events, video_name, &vid_config, &cam).unwrap();

    }

}






























