use clap::Parser;
use battleship_filter::filter::filter_and_count;

#[derive(Parser)]
#[command(name = "battleship-filter")]
#[command(about = "Filter and count ship hit frequencies from a board data file (supports zstd compression)", long_about = None)]
struct Cli {
    /// Path to the board data file (raw 16-byte masks, optionally zstd compressed). Use "-" to read from stdin.
    #[arg(short, long)]
    file: String,

    /// Hit mask as hex (e.g., 0xabcdef...)
    #[arg(long)]
    hit: String,

    /// Miss mask as hex
    #[arg(short, long)]
    miss: String,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let hit_mask = u128::from_str_radix(cli.hit.trim_start_matches("0x"), 16)
        .expect("Invalid hit mask hex");
    let miss_mask = u128::from_str_radix(cli.miss.trim_start_matches("0x"), 16)
        .expect("Invalid miss mask hex");

    let (counts, matched) = filter_and_count(&cli.file, hit_mask, miss_mask)?;

    eprintln!("Matched boards: {}", matched);
    // Print 9x9 grid of counts
    for y in 0..9 {
        for x in 0..9 {
            let idx = y * 9 + x;
            print!("{}{}", counts[idx], if x < 8 { "," } else { "" });
        }
        println!();
    }
    Ok(())
}
