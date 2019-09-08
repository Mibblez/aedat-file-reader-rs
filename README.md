# AEDAT File Reader Rs

Program for converting AEDAT files to CSV or video.

Rust port of this [UWP App](https://github.com/MartinNowak96/AEDAT-File-Reader).

### Installing

Ensure that the latest version of [Rust](https://www.rust-lang.org/tools/install) has been downloaded.

Build the project with:

```
git clone https://github.com/Mibblez/aedat-file-reader-rs
cd aedat-file-reader-rs
cargo build --release
```

### External Dependencies

For video exports:

* [Python3](https://www.python.org/downloads/)
* OpenCV
* natsort


## Basic Usage

CSV export:
```
aedat_reader csv <filename> <-i|-e> <-c|-p|-n>
```

* Use -i to include event polarity, -e to exclude event polarity
* Use -c to display coordinates as X and Y, -p to represent coordinates as pixel number, -n to exclude coordinate information


Video export:
```
aedat_reader vid <filename> --max_frames <maxFrames> --window_size <windowSize> <--time_based|--event_based>
```

* Use either time_based or event_based to set the frame reconstruction method
* Use window_size to indicate the duration of each frame (microseconds for time_based; number of events for event_based)
* Use max_frames to set a limit on the number of frames in the video



## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details
