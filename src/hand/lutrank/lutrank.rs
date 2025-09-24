// TODO: make this CLI better...
extern crate dealrs;

use std::{io::{BufWriter}, time::Instant};

use dealrs::hand::{lutrank::LutRank, Hand};

use clap::Parser;

/// Simulation program that simulates pocket occurrences in random deals
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {

    /// Path to save the lookup tables as a JSON file
    #[arg(short, long, default_value = "./src/hand/lutrank/lutrank.json")]
    path: String,

    /// If given, generate an additional output file with a human-readable markdown ranking file
    #[arg(short, long, default_value =  "./src/hand/lutrank/lutrank.md")]
    markdown: String,

    /// If given, reload the lookup table from the generated JSON string to test performance
    #[arg(short, long, default_value = "false")]
    reload: bool,

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // extract the path we are saving the lookup tables to
    let path = args.path;

    // now, create the lookup table from brute force computation
    println!("creating lookup table for 5-card hands via brute force computation ...");
    let timer = Instant::now();
    let lut = LutRank::from_brute_force();
    println!("lookup table created in {:.3?} ms", timer.elapsed().as_secs_f64() * 1000.0);

    // calculate estimated heap size
    let memuse = std::mem::size_of::<u16>() * (lut.allsuited.len() + lut.nonsuited.len()) + std::mem::size_of::<Hand>() * (lut.orders2hands.len());
    println!("memory usage of allsuited LUT: {:}", lut.allsuited.len() * std::mem::size_of::<u16>());
    println!("memory usage of nonsuited LUT: {:}", lut.nonsuited.len() * std::mem::size_of::<u16>());
    println!("memory usage orders2hands: {:}", lut.orders2hands.len() * std::mem::size_of::<Hand>());
    println!("estimated memory usage: {:} bytes", memuse);
    println!("writing output to file: {:} ...", path);

    println!("size of Hand: {:} bytes", std::mem::size_of::<Hand>());

    if false {
        let lut_str = serde_json::to_string_pretty(&lut)?;
        std::fs::write(&path, &lut_str)?;
    } else {
        let lut_str = serde_json::to_string(&lut)?;
        std::fs::write(&path, &lut_str)?;
    }
    
    let markdown = args.markdown;
    println!("writing text ranking to file: {:} ...", markdown);
    let file = std::fs::File::create(markdown)?;
    let mut writer = BufWriter::new(file);
    lut.write_markdown(&mut writer)?;

    // // now, output CBOR
    // let cbor_path = path.replace(".json", ".cbor");
    // println!("writing CBOR to file: {:} ...", cbor_path);
    // let file = std::fs::File::create(cbor_path)?;
    // let mut writer = BufWriter::new(file);
    // serde_cbor::to_writer(&mut writer, &lut)?;

    // if args.reload {
    //     println!("reloading lookup table from JSON string ...");
    //     let timer = Instant::now();
    //     let _ = serde_json::from_str::<LutBest5>(&lut_str)?;
    //     println!("lookup table reloaded in {:.3?} ms", timer.elapsed().as_secs_f64() * 1000.0);
    // }

    println!("done!");

    Ok(())
}
