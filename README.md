# AEDAT File Reader Rs

Program for converting AEDAT files to CSV or video.

Rust port of this [UWP App](https://github.com/MartinNowak96/AEDAT-File-Reader).

### Installing

Ensure that the latest version of [Rust](https://www.rust-lang.org/tools/install) has been installed.

Build the project with:

```
git clone https://github.com/Mibblez/aedat-file-reader-rs
cd aedat-file-reader-rs
cargo build --release
```

### External Dependencies

For video exports:

* [Python3](https://www.python.org/downloads/) with the following packages installed
    * opencv-python
    * natsort


## Basic Usage

CSV export:
```
aedat_reader csv <filename> <--coords|--pixel_number|--no_spatial> <--include_polarity|--exclude_polarity>
```

* Use --include_polarity (-i) to include event polarity, --exclude_polarity (-e) to exclude event polarity
* Use --coords (-c) to display coordinates as X and Y, --pixel_number (-p) to represent coordinates as pixel number, --exclude_polarity (-n) to exclude coordinate information

Time window CSV export:
```
aedat_reader time_windows <filename> --window_size <windowSize> 
```

* Use --max_frames (-m) to set a limit on the number of rows in the CSV
* Use --window_size (-w) to indicate the time covered by each time window in microseconds

Video export:
```
aedat_reader vid <filename> --max_frames <maxFrames> --window_size <windowSize> <--time_based|--event_based>
```

* Use one of either --time_based or --event_based flags to set the frame reconstruction method
* Use --window_size (-w) to indicate the duration of each frame (microseconds for time_based; number of events for event_based)
* Use --max_frames (-m) to set a limit on the number of frames in the video





## License

This project is licensed under the MIT License - see the [LICENSE.txt](LICENSE.txt) file for details
