//#![feature(is_sorted)]
use csv;
use std::{error::Error, fs::File, io::BufWriter, path::PathBuf, io};
use vcd::{ self, TimescaleUnit, SimulationCommand };
use regex::Regex;
use num_traits::PrimInt;

struct RigolDataSeries {
    timestamp: f64,
    signals: u16,
}

fn read_rigol_csv<R: io::Read>(io_reader: R) -> Result<Vec<RigolDataSeries>, Box<dyn Error>> {
	
    let mut rdr = csv::ReaderBuilder::new().flexible(true).from_reader(io_reader);

    // TODO: Handle CSV when timestamps are in each row (enum/option)
    // Initial timestamp...
    
    let t0;
    let t_inc;

	let mut i_csv = None;

	let mut i_d7_d0 = None;
	let mut i_d15_d8 = None;

	let mut i_d0_d7 = None;
	let mut i_d8_d15 = None;
    {
		let mut t0_option : Option<f64> = None; 
		let mut t_inc_option: Option<f64> = None;
		
		for (pos, header) in rdr.headers()?.iter().enumerate() {
			
			let t0_capture = Regex::new(r"t0 = ([^s]*)s$").unwrap().captures(header);
			if t0_capture.is_some() { t0_option = Some(t0_capture.unwrap()[1].parse::<f64>()?); continue; }
			
			let t_inc_capture = Regex::new(r"tInc = ([^s]*)").unwrap().captures(header);
			if t_inc_capture.is_some() { t_inc_option = Some(t_inc_capture.unwrap()[1].parse::<f64>()?); continue; }
			
			match header {
				""        =>  Ok(()),
				"Time(s)" =>  Ok(i_csv = Some(pos)),
				"D7-D0"   =>  Ok(i_d7_d0 = Some(pos)),
				"D15-D8"  =>  Ok(i_d15_d8 = Some(pos)),
				"D0-D7"   =>  Ok(i_d0_d7 = Some(pos)),
				"D8-D15"  =>  Ok(i_d8_d15 = Some(pos)),
				_ => Err("Unknown header {header}"),
			}?
		}
		t0 = t0_option.unwrap();
		t_inc = t_inc_option.unwrap();
	}
	
    
    println!("Initial timestamp {t0} with increments of {t_inc} seconds");

    let mut signals: Vec<RigolDataSeries> = vec![];

	let mut t_now = t0;
    for row in rdr.records() {
		
		let r = row?;

		t_now = t_now + t_inc;
        let t_csv = r[i_csv.unwrap()].parse::<f64>()?;
        
        assert!( (t_now - t_csv).abs() < t_inc);
                
        let mut d_all : u16 = 0;
        
        if i_d7_d0.is_some() { d_all += r[i_d7_d0.unwrap()].parse::<f64>()? as u16; }
        if i_d0_d7.is_some() { d_all += r[i_d0_d7.unwrap()].parse::<f64>()? as u16; }

        if i_d15_d8.is_some() { d_all += ( r[i_d15_d8.unwrap()].parse::<f64>()? as u16) << 8; }
        if i_d8_d15.is_some() { d_all += ( r[i_d8_d15.unwrap()].parse::<f64>()? as u16) << 8; }
        
        signals.push(RigolDataSeries { timestamp: t_csv, signals: d_all } );
    }
	
	println!("Signals read");

    Ok(signals)
}

fn write_vcd<W: io::Write>(io_writer: &mut W, sigs: Vec<RigolDataSeries>) -> Result<(), Box<dyn Error>> {

	fn vcd_vector_from_u16(d : u16) -> Vec<vcd::Value> {
		
		let mut out_bits = vec![];
		for n in 0..16 { out_bits.push( if d&(1<<(15-n))==0 { vcd::Value::V0 } else { vcd::Value::V1 } ) }
		return out_bits
	}
	
	let mut writer = vcd::Writer::new(io_writer);

	
    // Write the header
    writer.timescale(1, TimescaleUnit::US)?;
    writer.add_module("top")?;
    let data = writer.add_wire(16, "data")?;
    writer.upscope()?;
    writer.enddefinitions()?;
  
    // Write the initial values
    writer.begin(SimulationCommand::Dumpvars)?;
    writer.change_vector(data, &vcd_vector_from_u16(sigs[0].signals))?;
    writer.end()?;
  
    let offset = sigs[0].timestamp;
    // Write the data values
    for s in sigs {
      // TODO: Tweak that 10000000 with the defined timescale in the header
      writer.timestamp(((s.timestamp-offset).abs() * 1000000000.0) as u64)?;
      writer.change_vector(data,  &vcd_vector_from_u16(s.signals))?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let sigs = read_rigol_csv(io::stdin())?;
    write_vcd(&mut BufWriter::new(File::create(PathBuf::from("data/test.vcd"))?), sigs)?;
    Ok(())
}
