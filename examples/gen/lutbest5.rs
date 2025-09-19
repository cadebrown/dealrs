// examples/gen/lutbest5.rs - program that generates the lookup table for 5-card hands

extern crate dealrs;

use std::{io::{BufWriter}, time::Instant};

use dealrs::{hand::lutbest5::LutBest5};

use clap::Parser;

/// Simulation program that simulates pocket occurrences in random deals
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {

    /// Path to save the lookup tables as a JSON file
    #[arg(short, long, default_value = "./src/hand/lutbest5/lutbest5.json")]
    path: String,

    /// If given, generate an additional output file with a human-readable markdown ranking file
    #[arg(short, long, default_value =  "./src/hand/lutbest5/lutbest5.md")]
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
    let lut = LutBest5::from_brute_force();
    println!("lookup table created in {:.3?} ms", timer.elapsed().as_secs_f64() * 1000.0);

    // convert to a JSON string with serde, either pretty or minified
    let lut_str = serde_json::to_string(&lut)?;

    println!("writing output to file: {:} ...", path);
    std::fs::write(&path, &lut_str)?;
    
    let markdown = args.markdown;
    println!("writing text ranking to file: {:} ...", markdown);
    let file = std::fs::File::create(markdown)?;
    let mut writer = BufWriter::new(file);
    lut.write_markdown(&mut writer)?;

    if args.reload {
        println!("reloading lookup table from JSON string ...");
        let timer = Instant::now();
        let _ = serde_json::from_str::<LutBest5>(&lut_str)?;
        println!("lookup table reloaded in {:.3?} ms", timer.elapsed().as_secs_f64() * 1000.0);
    }

    println!("done!");

    Ok(())
}
