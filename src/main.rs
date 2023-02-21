//#![feature(is_sorted)]
use csv;
use std::{error::Error, fs::File, io::BufWriter, path::PathBuf, io};
use vcd::{ self, TimescaleUnit, SimulationCommand };
use regex::Regex;

struct RigolTSData {
	timestamp: f64,
	data: u16
}

struct RigolCSVData {
	
	t0 : f64,
    t_inc : f64,
	signals: Vec<RigolTSData>
}

fn read_rigol_csv<R: io::Read>(io_reader: R) -> Result<RigolCSVData, Box<dyn Error>> {
	
    let mut rdr = csv::ReaderBuilder::new().flexible(true).from_reader(io_reader);

    // TODO: Handle CSV when timestamps are in each row (enum/option)
    // Initial timestamp...
    
    let mut rigol = RigolCSVData {
		t0 : 0.0,
		t_inc : 0.0,
		signals : vec![]
	};
    
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
		rigol.t0 = t0_option.unwrap();
		rigol.t_inc = t_inc_option.unwrap();
	}
	
    
    println!("Initial timestamp {} with increments of {} seconds", rigol.t0, rigol.t_inc);

	let mut async_count = 0;
	
    for (i, row) in rdr.records().enumerate() {
		
		let r = row?;

		let t_now = rigol.t0 + ((i) as f64) * rigol.t_inc;
        let t_csv = r[i_csv.unwrap()].parse::<f64>()?;
        
        //println!("{} == {}", format!("{:+0.2e}", t_now), format!("{:+0.2e}", t_csv));
        async_count = async_count + 1;
		if format!("{:+0.4e}", t_now) == format!("{:+0.4e}", t_csv) { async_count = 0; }
		assert!(async_count < 5);
        
        let mut d_all : u16 = 0;
        
        if i_d7_d0.is_some() { d_all += r[i_d7_d0.unwrap()].parse::<f64>()? as u16; }
        if i_d0_d7.is_some() { d_all += r[i_d0_d7.unwrap()].parse::<f64>()? as u16; }

        if i_d15_d8.is_some() { d_all += ( r[i_d15_d8.unwrap()].parse::<f64>()? as u16) << 8; }
        if i_d8_d15.is_some() { d_all += ( r[i_d8_d15.unwrap()].parse::<f64>()? as u16) << 8; }
        
        rigol.signals.push(RigolTSData { timestamp: t_now, data: d_all } );
    }
	
	println!("Signals read");

    Ok(rigol)
}

fn write_vcd<W: io::Write>(io_writer: &mut W, rigol: RigolCSVData) -> Result<(), Box<dyn Error>> {

	fn vcd_vector_from_u16(d : u16) -> Vec<vcd::Value> {
		
		let mut out_bits = vec![];
		for n in 0..16 { out_bits.push( if d&(1<<(15-n))==0 { vcd::Value::V0 } else { vcd::Value::V1 } ) }
		return out_bits
	}
	
	let mut writer = vcd::Writer::new(io_writer);

	
    // Write the header
    writer.timescale( (rigol.t_inc / 1e-12) as u32, TimescaleUnit::PS)?;
    writer.add_module("top")?;
    let data = writer.add_wire(16, "data")?;
    writer.upscope()?;
    writer.enddefinitions()?;
  
    // Write the initial values
    writer.begin(SimulationCommand::Dumpvars)?;
    writer.change_vector(data, &vcd_vector_from_u16(rigol.signals[0].data))?;
    writer.end()?;
  
    let offset = rigol.signals[0].timestamp;
    // Write the data values
    for s in rigol.signals {
      // TODO: Tweak that 10000000 with the defined timescale in the header
      writer.timestamp(((s.timestamp-offset).abs() * 1000000000.0) as u64)?;
      writer.change_vector(data,  &vcd_vector_from_u16(s.data))?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let rigol_data = read_rigol_csv(io::stdin())?;
    write_vcd(&mut BufWriter::new(File::create(PathBuf::from("data/test.vcd"))?), rigol_data)?;
    Ok(())
}
