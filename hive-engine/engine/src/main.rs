use hive_library::*;

fn main() {
    let (config, args) = configure_player().unwrap();
    match args.first().unwrap_or(&"uhp".to_owned()).as_ref() {
        "uhp" => {
            uhp_serve(config);
        }
        _ => panic!("Unrecognized command: {}", args.join(" ")),
    }
}
