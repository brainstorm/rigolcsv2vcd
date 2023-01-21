use csv;
use serde::{self, Deserialize};
use std::error::Error;

use embedded_hal_vcd::{self, writer::VcdWriterBuilder};
use embedded_time::duration::Extensions;
use std::fs::File;
use std::io::{self, BufWriter};

#[derive(Debug, Deserialize)]
struct RigolCSV {
    #[serde(rename = "Time(s)")]
    timestamp: String,
    #[serde(rename = "D7-D0")]
    d7_d0: String, // TODO: Unfortunately those fields are "user-flippable" in order from the scope, i.e: d0_d7 vs d7_d0
    #[serde(rename = "D15-D8")]
    d15_d8: String,
}

fn read_rigol_csv() -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true) // ignore broken header
        .from_reader(io::stdin());

    // TODO: Handle CSV when timestamps are in each row (enum/option)
    // Initial timestamp...
    let header = rdr.headers()?.clone();
    dbg!(&header);    
    let t0_header: Vec<&str> = header[3].split('=').collect();
    let t0 = t0_header[1].trim_start().replace('s', "").parse::<f32>()?;
    // ...and increments
    let tinc_header: Vec<&str> = header[4].split('=').collect();
    let tinc = tinc_header[1].trim_start().parse::<f32>()?;
    println!("Initial timestamp {t0} with increments of {tinc} seconds");

    let mut t_now: f32;
    let mut t_csv: f32;

    for row in rdr.deserialize().skip(1) {
        t_now = t0 + tinc;
        let record: RigolCSV = row?;
        t_csv = record.timestamp.parse::<f32>()?;
        dbg!(t_now);
        dbg!(t_csv);

        assert_eq!(t_now, t_csv);
        //break;
    }
    // Now do the splitting of wires from its Dx-Dy "bundles"
    //let mut timestamp = t0;
    // for result in rdr.deserialize().skip(1) {
    //     let row: OrigOscilloscope = result?;

    //     // if rdr.position().line().rem_euclid(8) {
    //     // // Read D0-D15 field(s), expanding them into the current row, matching its column
    //     // }
    //     // println!("{:#?}", row.d7_d0.parse::<f32>()?);

    //     // update timestamp for this row
    //     timestamp = timestamp + tinc;
    // }
    Ok(())
}

fn write_vcd() -> Result<(), std::io::Error> {
    let f2 = BufWriter::new(File::create("data/test2.vcd")?);
    let mut writer = VcdWriterBuilder::new(f2)?.build()?;

    writer.timestamp(1_u32.nanoseconds())?;
    writer.sample()?;

    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    read_rigol_csv().unwrap();
    //write_vcd().unwrap();
    Ok(())
}
