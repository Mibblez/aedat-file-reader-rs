# AEDAT File Reader Rs

Program for converting AEDAT files to CSV or video.

Rust port of this [UWP App](https://github.com/MartinNowak96/AEDAT-File-Reader).

### Installing

Ensure that the latest version of [Rust](https://www.rust-lang.org/tools/install) has been downloaded.

Build the project with:

```
git clone https://github.com/Mibblez/aedat_file_reader_rs
cd aedat_file_reader_rs
cargo build --release
```

### External Dependencies

For video exports:

* [Python3](https://www.python.org/downloads/)
* OpenCV


## Usage

CSV export:
```
aedat_reader csv <filename> <-i|-e> <-c|-p|-n>
```

* Use -i to include event polarity, -e to exclude event polarity
* Use -i to display coordinates as X and Y, -p to represent coordinates as pixel number, -n to exclude coordinate information


Video export:
```
aedat_reader vid <filename> --frame_time <frameTime> --max_frames <maxFrames>
```

* Use time_per_frame to indicate the duration of each frame in microseconds
* Use max_frames to set a limit on the number of frames in the video



## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details
