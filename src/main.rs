#[macro_use]
extern crate clap;

mod lib;

pub use crate::lib::aedat_utilities;

use std::io::prelude::*;
use std::process;
use std::fs::File;
use std::path::Path;
use clap::{App, Arg, SubCommand, ArgGroup, ArgMatches};

fn csv_convert(args: &ArgMatches) {
    let csv_config = aedat_utilities::CsvConfig::new(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments\n{}", err);
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

fn vid_convert(args: &ArgMatches) {
    let vid_config = aedat_utilities::VidConfig::new(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments\n{}", err);
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

    if args.is_present("timeBasedReconstruction") {
        aedat_utilities::create_time_based_video(events, video_name, &vid_config, &cam).unwrap();
    } else {
        aedat_utilities::create_event_based_video(events, video_name, &vid_config, &cam).unwrap();
    }
}

fn main() {

    let matches = App::new("aedat_reader")
        .about("Program for converting AEDAT files to CSV or video.")
        .author(crate_authors!())
        .subcommand(SubCommand::with_name("csv")
            .about("Exports AEDAT to CSV")
            .arg(Arg::with_name("filename")
                .help("The AEDAT file to be processed")
                .required(true)
            )
            .groups(&[
                ArgGroup::with_name("csv_spatial")
                    .args(&["coords", "pixelNumber", "noSpatial"])
                    .required(true),
                ArgGroup::with_name("csv_polarity")
                    .args(&["includePolarity", "excludePolarity"])
                    .required(true)
            ])
            .arg(Arg::with_name("coords")
                .help("Represents coordinates as X and Y columns")
                .short("c")
                .long("coords")
            )
            .arg(Arg::with_name("pixelNumber")
                .help("Represents coordinates as a single column of values")
                .short("p")
                .long("pixel_number")
            )
            .arg(Arg::with_name("noSpatial")
                .help("No spatial information")
                .short("n")
                .long("no_spatial")
            )
            .arg(Arg::with_name("includePolarity")
                .help("Includes polarity")
                .short("i")
                .long("include_polarity")
            )
            .arg(Arg::with_name("excludePolarity")
                .help("Excludes polarity")
                .short("e")
                .long("exclude_polarity")
            )
        )
        .subcommand(SubCommand::with_name("vid")
            .about("Exports AEDAT to AVI video")
            .arg(Arg::with_name("filename")
                .help("The AEDAT file to be processed")
                .required(true)
            )
            .groups(&[
                ArgGroup::with_name("reconstructionMethod")
                    .args(&["timeBasedReconstruction", "eventBasedReconstruction"])
                    .required(true),
            ])
            .arg(Arg::with_name("timeBasedReconstruction")
                .help("Reconstruct frames based on a fixed time window")
                .long("time_based")
            )
            .arg(Arg::with_name("eventBasedReconstruction")
                .help("Reconstruct frames based on a fixed number of events")
                .long("event_based")
            )
            .arg(Arg::with_name("windowSize")
                .help("The duration of each frame. Microseconds for time based reconstruction; number of events for event based reconstruction")
                .takes_value(true)
                .short("w")
                .long("window_size")
                .required(true)
            )
            .arg(Arg::with_name("maxFrames")
                .help("The maximum number of frames to be encoded")
                .takes_value(true)
                .short("m")
                .long("max_frames")
            )
            .arg(Arg::with_name("excludeOffEvents")
                .help("Exclude off events in the exported video")
                .long("exclude_off")
                .conflicts_with("excludeOnEvents")
            )
            .arg(Arg::with_name("excludeOnEvents")
                .help("Exclude on events in the exported video")
                .long("exclude_on")
                .conflicts_with("excludeOffEvents")
            )
            .arg(Arg::with_name("keepFrames")
                .help("Keep the reconstructed frames")
                .short("k")
                .long("keep_frames")
            )
            .arg(Arg::with_name("omitVideo")
                .help("Do not reconstruct a video")
                .short("o")
                .long("omit_video")
                .requires("keepFrames")
            )
        )
        .get_matches();

    match matches.subcommand() {
        ("csv", Some(csv_matches)) => csv_convert(csv_matches),
        ("vid", Some(vid_matches)) => vid_convert(vid_matches),
        _ => println!("Subcommand 'vid' or 'csv' required."),
    }

}






























