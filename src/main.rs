use std::process;
use std::env;

mod lib;
pub use crate::lib::aedat_utilities;

fn main() {
    // Get environment variables
    let args: Vec<String> = env::args().collect();
    let config = aedat_utilities::Config::new(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    let header_end =  aedat_utilities::find_header_end(&config.filename).unwrap();
    println!("End of header at position: {:?}", header_end);

    let events = aedat_utilities::get_events(header_end, &config.filename).unwrap();

    println!("Total number of events: {}", events.len());

    use std::time::Instant;
    let now = Instant::now();

    aedat_utilities::create_csv(events, "test.csv", &config).unwrap();

    let elapsed = now.elapsed();
    let sec = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
    println!("Export time: {} seconds", sec);
}

































