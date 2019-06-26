pub mod aedat_utilities {
    use std::io::prelude::*;
    use std::fs::File;
    use std::convert::TryInto;
    use std::io::ErrorKind;

    pub struct Event {
        pub bytes: [u8; 8],
    }

    impl Event {
        pub fn get_polarity(&self) -> bool {
            //Event polarity is located in the fourth bit of the third byte
            (self.bytes[2] >> 3 & 1) == 1
        }

        pub fn get_timestamp(&self) -> i32 {
            // Timestamp is found in the last four bytes
            (((self.bytes[7] as u32) << 0) +
                ((self.bytes[6] as u32) << 8) +
                ((self.bytes[5] as u32) << 16) +
                ((self.bytes[4] as u32) << 24)) as i32
        }

        pub fn get_coords_dvs128(&self) -> (u8, u8) {
            // DVS128   (X = width - bits33-39 ) ; (Y = height - bits40-46 ) [bytes 2-3]
            (128 - ((self.bytes[3] >> 1) & 0b1111111) as u8, // X coordinate
             128 - (self.bytes[2] & 0b1111111) as u8)        // Y coordinate
        }

        // DVS240C  (X = width - bits44-53) ; (Y = height - bits54-62) [bytes 0-2]
        // Make method pls
    }

    pub struct Config {
        pub filename: String,
        pub include_polarity: bool,
    }

    impl Config {
        pub fn new(args: &[String]) -> Result<Config, std::io::Error> {
            if args.len() < 2 {
                return Err(std::io::Error::new(ErrorKind::InvalidInput,
                                               "Not enough arguments"));
            }

            let filename = args[1].clone();
            let include_polarity = match args[2].as_ref() {
                "-p" => true,
                "-np" => false,
                _ => return Err(std::io::Error::new(ErrorKind::InvalidInput,
                                                    "Invalid input"))
            };

            Ok(Config { filename, include_polarity })
        }
    }

    pub fn find_header_end(file_path: &str) -> Result<u32, std::io::Error> {
        let mut f = File::open(file_path)?;

        // Equivalent to: #End Of ASCII
        const END_OF_ASCII: [u8; 22] = [35, 69, 110, 100, 32, 79, 102, 32, 65, 83, 67, 73, 73, 32, 72, 101, 97, 100, 101, 114, 13, 10];

        let mut header_end_q = Vec::with_capacity(END_OF_ASCII.len());
        let mut header_end_index = 0;

        // read up to 0.5MB
        const HALF_MB: u32 = 524288;
        let mut buffer = [0; 524288];
        f.read(&mut buffer)?;

        let mut iter = buffer.iter();

        loop {
            let next = iter.next();
            match next {
                Some(num) => header_end_q.push(num.to_owned()),
                None => break,
            }

            // Pop oldest value off the queue if it becomes too large
            if header_end_q.len() > END_OF_ASCII.len() {
                header_end_q.remove(0);
            }

            header_end_index += 1;

            // Break if end of header has been found
            if &END_OF_ASCII[..] == &header_end_q[..] { break; }

            if header_end_index == HALF_MB - 1 {
                return Err(std::io::Error::new(ErrorKind::NotFound, "End of header not found"));
            }
        }
        Ok(header_end_index)
    }


    pub fn get_events(end_of_header_index: u32, file_path: &str) -> Result<Vec<Event>, std::io::Error> {
        // Size of an event in bytes
        const EVENT_SIZE: usize = 8;

        // Read file
        let mut f = File::open(file_path)?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)?;

        // Skip over the header to get directly to the event data
        let mut aedat_iter = buffer.iter().skip(end_of_header_index as usize);

        // Pre-allocate space in vec for all events
        let mut events = Vec::with_capacity(aedat_iter.len() / EVENT_SIZE);
        let mut bytes_tmp = Vec::with_capacity(EVENT_SIZE);

        // Loop over all bytes in file
        loop {
            let next = match aedat_iter.next() {
                Some(t) => t,
                None => break,  // Exit loop if end of file is reached
            };

            bytes_tmp.push(next.to_owned());

            // Collected enough bytes for an event. Create event and push to Vec of events
            if bytes_tmp.len() == EVENT_SIZE {
                let event = Event {
                    bytes: bytes_tmp[..].try_into().
                        expect("Slice with incorrect length")
                };
                bytes_tmp.clear();
                events.push(event);
            }
        }
        Ok(events)
    }

    fn format_polarity(polarity: bool) -> String {
        format!("{},", match polarity {
            true => "1",
            false => "-1"
        })
    }

    // TODO: Make more modular
    fn config_header(config: &Config) -> &[u8] {
        if config.include_polarity == true {
            return "On/Off,X,Y,Timestamp\n".as_bytes();
        } else {
            return "X,Y,Timestamp\n".as_bytes();
        }
    }

    pub fn create_csv(events: Vec<Event>, filename: &str, config: &Config) -> std::io::Result<()> {
        // Create CSV file and write header
        let mut new_csv = File::create(filename)?;
        let csv_header = config_header(&config);
        new_csv.write(csv_header)?;

        // Create write buffer and preallocate space
        const BUF_PREALLOCATE_SIZE: usize = 100000;
        let mut write_buf = Vec::with_capacity(BUF_PREALLOCATE_SIZE);

        for event in events {
            let (x, y) = event.get_coords_dvs128();

            write!(&mut write_buf, "{}",
                   format!("{p}{xy}{t}\n",
                           p = match config.include_polarity {
                               true => format_polarity(event.get_polarity()),
                               false => String::from(""),
                           },
                           xy = format!("{x},{y},", x = x, y = y),
                           t = event.get_timestamp()))?;

            // Write events to disk once enough have been collected
            if write_buf.len() >= BUF_PREALLOCATE_SIZE {
                new_csv.write(write_buf.as_slice())?;
                write_buf.clear();
            }
        }

        // Write any remaining events to disk
        if !write_buf.is_empty() {
            new_csv.write(write_buf.as_slice())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::aedat_utilities::*;

    #[test]
    fn event_test_dvs128() {
        let test_event_bytes: [u8; 8] = [0, 0, 56, 231, 156, 86, 232, 205];
        let test_event_struct = Event { bytes: test_event_bytes };

        // Get event polarity
        let polarity = test_event_struct.get_polarity();
        assert_eq!(polarity, true);

        // Get timestamp
        let timestamp = test_event_struct.get_timestamp();
        assert_eq!(timestamp, -1672025907);

        // Get XY coordinates
        let (x, y) = test_event_struct.get_coords_dvs128();
        assert_eq!(x, 13);
        assert_eq!(y, 72);
    }
}