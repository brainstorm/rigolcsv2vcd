use csv;
use serde::{self, Deserialize};
use std::error::Error;

use embedded_hal_vcd::{self, writer::VcdWriterBuilder};
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

fn _parse_la_signal_group(group1: f32, group2: f32) {
    unimplemented!()
}

fn read_rigol_csv() -> Result<Vec<(u64, u32)>, Box<dyn Error>> {
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
    let mut t_csv: u64;

    let mut signals: Vec<(u64, u32)> = vec![];

    for row in rdr.deserialize().skip(1) {
        let record: RigolCSV = row?;
        // Compare t0+tinc vs timestamp divergence
        t_now = t0 + tinc;
        t_csv = record.timestamp.parse::<u64>()?;
        // dbg!(t_now);
        // dbg!(t_csv);
        // Parse digital signal groups
        let d_group_low = record.d7_d0.parse::<f32>()?.to_bits();
        let d_group_high = record.d15_d8.parse::<f32>()?.to_bits();

        let d_all = (d_group_high << 8) + d_group_low;
        signals.push((t_csv, d_all));
        //assert_eq!(t_now, t_csv);
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
    Ok(signals)
}

// fn _write_hal_vcd(signals: Vec<(u64, u32)>) -> Result<(), std::io::Error> {
//     let f2 = BufWriter::new(File::create("data/test2.vcd")?);
//     let mut writer = VcdWriterBuilder::new(f2)?;
//     let mut apin = writer.add_push_pull_pin("reference")?;

//     let mut writer = writer.build()?;

//     for signal in signals {
//         let mut timestamp = signal.0;
//         writer.timestamp(timestamp);

//         if signal.1 {
//             apin.set_high()?;
//         } else {
//             apin.set_low()?;
//         }

//         //writer.timestamp(timestamp)?;
//         writer.sample()?;
//     }

//     Ok(())
// }

fn main() -> Result<(), std::io::Error> {
    let sigs = read_rigol_csv().unwrap();
    //write_hal_vcd(sigs).unwrap();
    Ok(())
}
