use hive_library::generate_games;
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = pico_args::Arguments::from_env();
    let no_games: usize = args.value_from_str("--games")?;
    let min_rounds: usize = args.value_from_str("--min-rounds")?;
    let max_rounds: usize = args.value_from_str("--max-rounds")?;
    generate_games(no_games, min_rounds, max_rounds, io::stdout())?;
    Ok(())
}
