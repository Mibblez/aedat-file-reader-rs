use clap::ArgMatches;
use std::path::PathBuf;

pub enum CoordMode {
    NoCoord,
    XY,
    PixelNum,
}

pub struct CsvConfig {
    pub filename: PathBuf,
    pub include_polarity: bool,
    pub coords: CoordMode,
    pub offset_time: bool,
}

impl CsvConfig {
    pub fn new(args: &ArgMatches) -> Result<CsvConfig, std::io::Error> {
        let mut filename = args.get_one::<PathBuf>("filename").unwrap().clone();
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

pub struct TimeWindowConfig {
    pub filename: PathBuf,
    pub include_both_column: bool,
    pub include_pgm: bool,
    pub window_size: u32,
    pub max_windows: u32,
    pub pgm_scale: usize,
    pub pgm_threshold: usize,
}

impl TimeWindowConfig {
    pub fn new(args: &ArgMatches) -> Result<TimeWindowConfig, std::io::Error> {
        let mut filename = args.get_one::<PathBuf>("filename").unwrap().clone();
        filename.set_extension("csv");

        let window_size = args.get_one::<u32>("windowSize").unwrap().to_owned();

        let max_windows: u32 = match args.get_one::<u32>("maxWindows") {
            Some(v) => v.to_owned(),
            None => std::u32::MAX,
        };

        let include_both_column = args.get_flag("includeBoth");

        let include_pgm = args.get_flag("includePgm");
        let pgm_scale = match args.get_one::<usize>("pgmScale") {
            Some(v) => v.to_owned(),
            None => 1,
        };
        let pgm_threshold = match args.get_one::<usize>("pgmThreshold") {
            Some(v) => v.to_owned(),
            None => 0,
        };

        Ok(TimeWindowConfig {
            filename,
            include_both_column,
            include_pgm,
            window_size,
            max_windows,
            pgm_scale,
            pgm_threshold,
        })
    }

    #[must_use]
    pub fn create_csv_header(&self) -> String {
        let mut header_tmp = String::from("On,Off");

        if self.include_both_column {
            header_tmp.push_str(",Both");
        }

        if self.include_pgm {
            header_tmp.push_str(",PGM_String");
        }

        header_tmp.push('\n');
        header_tmp
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
        let mut filename = args.get_one::<PathBuf>("filename").unwrap().clone();
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
