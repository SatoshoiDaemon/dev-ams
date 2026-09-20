fn main() {
    let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    match ams::initialize(root) {
        Ok(state) => println!("A Magic Sovereign — foundation ready (seed {})", state.seed),
        Err(error) => { eprintln!("A Magic Sovereign failed to start: {error}"); std::process::exit(1); }
    }
}
