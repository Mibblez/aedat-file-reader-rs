mod aedat_data;
mod aedat_header_tools;
mod cli_configs;
mod tests;

mod aedat_conversions;

use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process;
use std::time::Instant;

use clap::{Arg, ArgAction, ArgGroup, ArgMatches, Command};

use crate::aedat_data::get_events;
use crate::aedat_header_tools::{find_header_end, parse_camera_type};
use crate::cli_configs::*;
use aedat_conversions::csv::create_csv;
use aedat_conversions::time_window_csv::*;
use aedat_conversions::video::{create_event_based_video, create_time_based_video};

fn csv_convert(args: &ArgMatches) {
    let csv_config = CsvConfig::new(args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments\n{err}");
        process::exit(1);
    });

    // Read file
    let aedat_filename = args.get_one::<PathBuf>("filename").unwrap();

    let mut f = File::open(aedat_filename).unwrap();
    let mut aedat_file = Vec::new();
    f.read_to_end(&mut aedat_file).unwrap();

    let cam = parse_camera_type(&aedat_file).unwrap();
    let header_end = find_header_end(&aedat_file).unwrap();
    let events = get_events(header_end, &aedat_file).unwrap();

    let now = Instant::now();

    create_csv(events, &csv_config, &cam).unwrap();

    let elapsed = now.elapsed();
    let sec = (elapsed.as_secs() as f64) + (f64::from(elapsed.subsec_nanos()) / 1_000_000_000.0);
    println!("Export time: {sec} seconds");
}

fn vid_convert(args: &ArgMatches) {
    let vid_config = VidConfig::new(args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments\n{err}");
        process::exit(1);
    });

    // Read file
    let aedat_filename = args.get_one::<PathBuf>("filename").unwrap();
    let mut f = File::open(aedat_filename).unwrap();
    let mut aedat_file = Vec::new();
    f.read_to_end(&mut aedat_file).unwrap();

    let cam = parse_camera_type(&aedat_file).unwrap();

    let header_end = find_header_end(&aedat_file).unwrap();

    let events = get_events(header_end, &aedat_file).unwrap();

    if args.get_flag("timeBasedReconstruction") {
        create_time_based_video(events, &vid_config, &cam).unwrap();
    } else {
        create_event_based_video(events, &vid_config, &cam).unwrap();
    }
}

fn time_window_convert(args: &ArgMatches) {
    let time_window_config = TimeWindowConfig::new(args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments\n{err}");
        process::exit(1);
    });

    // Read file
    let aedat_filename = args.get_one::<PathBuf>("filename").unwrap();
    let mut f = File::open(aedat_filename).unwrap();
    let mut aedat_file = Vec::new();
    f.read_to_end(&mut aedat_file).unwrap();

    let cam = parse_camera_type(&aedat_file).unwrap();

    let header_end = find_header_end(&aedat_file).unwrap();

    let events = get_events(header_end, &aedat_file).unwrap();

    create_time_window_csv(events, &time_window_config, &cam).unwrap();
}

fn main() {
    let matches = Command::new("aedat_reader")
        .about("Program for converting AEDAT files to CSV or video.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("csv")
                .long_flag("csv")
                .about("Export AEDAT to CSV")
                .arg(
                    Arg::new("filename")
                        .value_parser(clap::value_parser!(PathBuf))
                        .action(ArgAction::Set)
                        .required(true)
                        .help("Path to the AEDAT file to be processed"),
                )
                .group(
                    ArgGroup::new("csv_spatial")
                        .required(true)
                        .args(["coords", "pixelNumber", "noSpatial"])
                )
                .group(
                    ArgGroup::new("csv_polarity")
                        .required(true)
                        .args(["includePolarity", "excludePolarity"])
                )
                .arg(
                    Arg::new("coords")
                        .short('c')
                        .long("coords")
                        .action(ArgAction::SetTrue)
                        .help("Represent coordinates as X and Y columns")
                )
                .arg(
                    Arg::new("pixelNumber")
                        .short('p')
                        .long("pixel_number")
                        .action(ArgAction::SetTrue)
                        .help("Represent coordinates as a single column of values")
                )
                .arg(
                    Arg::new("noSpatial")
                        .short('n')
                        .long("no_spatial")
                        .action(ArgAction::SetTrue)
                        .help("Do not include spatial information")
                )
                .arg(
                    Arg::new("includePolarity")
                        .short('i')
                        .long("include_polarity")
                        .action(ArgAction::SetTrue)
                        .help("Includes polarity column")
                )
                .arg(
                    Arg::new("excludePolarity")
                        .short('e')
                        .long("exclude_polarity")
                        .action(ArgAction::SetTrue)
                        .help("Excludes polarity column")
                )
                .arg(
                    Arg::new("offsetTime")
                        .short('o')
                        .long("offset_time")
                        .action(ArgAction::SetTrue)
                        .help("Start timestamps in the exported csv at 0")
                ),
        )
        .subcommand(
            Command::new("vid")
                .long_flag("vid")
                .about("Export AEDAT to AVI video")
                .arg(
                    Arg::new("filename")
                        .value_parser(clap::value_parser!(PathBuf))
                        .action(ArgAction::Set)
                        .required(true)
                        .help("Path to the AEDAT file to be processed"),
                )
                .group(
                    ArgGroup::new("reconstructionMethod")
                        .args(["timeBasedReconstruction", "eventBasedReconstruction"])
                        .required(true)
                )
                .arg(
                    Arg::new("timeBasedReconstruction")
                        .long("time_based")
                        .action(ArgAction::SetTrue)
                        .help("Reconstruct frames based on a fixed time window"),
                )
                .arg(
                    Arg::new("eventBasedReconstruction")
                        .long("event_based")
                        .action(ArgAction::SetTrue)
                        .help("Reconstruct frames based on a fixed number of events"),
                )
                .arg(
                    Arg::new("windowSize")
                        .long("window_size")
                        .short('w')
                        .value_parser(clap::value_parser!(usize))
                        .action(ArgAction::Set)
                        .required(true)
                        .help("The duration of each frame. Microseconds for time based reconstruction; number of events for event based reconstruction"),
                )
                .arg(
                    Arg::new("maxFrames")
                        .long("max_frames")
                        .short('m')
                        .value_parser(clap::value_parser!(usize))
                        .action(ArgAction::Set)
                        .help("The maximum number of frames to be encoded"),
                )
                .arg(
                    Arg::new("excludeOffEvents")
                        .long("exclude_off")
                        .conflicts_with("excludeOnEvents")
                        .action(ArgAction::SetTrue)
                        .help("Exclude off events in the exported video")
                )
                .arg(
                    Arg::new("excludeOnEvents")
                        .long("exclude_on")
                        .conflicts_with("excludeOffEvents")
                        .action(ArgAction::SetTrue)
                        .help("Exclude on events in the exported video")
                )
                .arg(
                    Arg::new("keepFrames")
                        .long("keep_frames")
                        .short('k')
                        .action(ArgAction::SetTrue)
                        .help("Keep the reconstructed frames"),
                )
                .arg(
                    Arg::new("omitVideo")
                        .long("omit_video")
                        .short('o')
                        .action(ArgAction::SetTrue)
                        .help("Do not compile the reconstructed frames into a video"),
                ),
        ).subcommand(Command::new("time_windows")
            .long_flag("time_windows")
            .about("Export AEDAT to a series of time windows")
            .arg(
                Arg::new("filename")
                    .value_parser(clap::value_parser!(PathBuf))
                    .action(ArgAction::Set)
                    .required(true)
                    .help("Path to the AEDAT file to be processed"),
            )
            .arg(
                Arg::new("windowSize")
                    .long("window_size")
                    .short('w')
                    .value_parser(clap::value_parser!(u32))
                    .action(ArgAction::Set)
                    .required(true)
                    .help("The duration of each window in microseconds"),
            )
            .arg(
                Arg::new("maxWindows")
                    .long("max_windows")
                    .short('m')
                    .value_parser(clap::value_parser!(u32))
                    .action(ArgAction::Set)
                    .help("The maximum number time windows to be encoded"),
            )
            .arg(
                Arg::new("includeBoth")
                    .long("include_both")
                    .short('i')
                    .action(ArgAction::SetTrue)
                    .help("Include a column containing the sum of the ON and OFF events in a given time window"),
            )
            .arg(
                Arg::new("includePgm")
                    .long("include_pgm")
                    .short('p')
                    .requires("pgmThreshold")
                    .requires("pgmScale")
                    .action(ArgAction::SetTrue)
                    .help("Include a downscaled PGM image in each row. \
                           Note that these PGM images will contain dashes in place of newline characters. \
                           The dashes will need to be converted to newlines before the images can be opened"),
            )
            .arg(
                Arg::new("pgmThreshold")
                    .long("pgm_threshold")
                    .short('t')
                    .requires("includePgm")
                    .value_parser(clap::value_parser!(usize))
                    .action(ArgAction::Set)
                    .help("The number of events required to activate a pixel in the downscaled PGM image"),
            )
            .arg(
                Arg::new("pgmScale")
                    .long("pgm_scale")
                    .short('s')
                    .requires("includePgm")
                    .value_parser(clap::value_parser!(usize))
                    .action(ArgAction::Set)
                    .help("The factor at which the downscaled image is scaled by"),
            ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("csv", csv_matches)) => csv_convert(csv_matches),
        Some(("vid", vid_matches)) => vid_convert(vid_matches),
        Some(("time_windows", time_windows_matches)) => time_window_convert(time_windows_matches),
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable
    }

    // match matches.subcommand() {
    //     Some(("csv", csv_matches)) => {
    //         if csv_matches.contains_id("filename") {
    //             let aedat_filename = csv_matches
    //                 .get_one::<PathBuf>("filename")
    //                 .unwrap()
    //                 .display();

    //             println!("Reading file {}...", aedat_filename);
    //             println!("Coords set: {}", csv_matches.get_flag("coords"));
    //             println!("No spatial set: {}", csv_matches.get_flag("noSpatial"));
    //             return;
    //         }
    //     }
    //     Some(("vid", vid_matches)) => {
    //         println!("Vid subcommand used");

    //         if vid_matches.contains_id("filename") {
    //             let aedat_filename = vid_matches
    //                 .get_one::<PathBuf>("filename")
    //                 .unwrap()
    //                 .display();
    //             println!("Reading file {}...", aedat_filename);

    //             let window_size = vid_matches.
    //                 get_one::<usize>("windowSize")
    //                 .unwrap();
    //             println!("Window size: {}", window_size)
    //         }
    //     }
    //     _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable
    // }
}
