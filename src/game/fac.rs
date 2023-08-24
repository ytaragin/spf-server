use csv::ReaderBuilder;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

fn read_csv_file(filename: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(filename)?;
    let mut reader = ReaderBuilder::new().has_headers(true).from_reader(file);

    for result in reader.records() {
        let record = result?;
        println!("{:?}", record);
    }

    Ok(())
}
