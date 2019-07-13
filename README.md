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

### Dependencies

For video exports:

* [Python3](https://www.python.org/downloads/)
* OpenCV


## Usage

CSV export:
```
cargo run --release [filename] -csv [-p|-np] [-xy|-pn|-nc]
```

* Use -p to include event polarity, -np to exclude event polarity
* Use -xy to display coordinates, -pn to represent coordinates as pixel number, -nc to exclude coordinate information


Video export:
```
cargo run --release [filename] -vid [time_per_frame] [max_frames]
```

* Use time_per_frame to indicate the duration of each frame in microseconds
* Use max_frames to set a limit on the number of frames in the video



## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details
