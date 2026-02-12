use hive_library::generate_games;
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    generate_games(10000, 50, 100, io::stdout())?;
    Ok(())
}
