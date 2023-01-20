use embedded_hal_vcd::{self, writer::VcdWriterBuilder};
//use embedded_hal::digital::{embedded_hal_vcd::pins::InputPin};
use embedded_time::duration::units;
use std::fs::File;
use std::io::{BufWriter};

fn main() -> Result<(), std::io::Error> {
    // construct a writer builder
    let f2 = BufWriter::new(File::create("data/test2.vcd")?);
    let writer = VcdWriterBuilder::new(f2).unwrap();
    // add output pin to writer
    // build the writer
    let mut writer = writer.build().unwrap();

    // get first timestamp from vcd and pass it to the writer
    writer.timestamp(units::Nanoseconds::new(1u64))?;

    writer.sample()?;
    writer.timestamp(units::Nanoseconds::new(1u64))?;

    Ok(())
}

