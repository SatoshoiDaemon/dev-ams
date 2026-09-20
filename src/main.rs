fn main() {
    let root = std::env::current_exe()
        .ok()
        .and_then(|executable| executable.parent().map(std::path::Path::to_path_buf))
        .filter(|directory| directory.join("config.toml").is_file())
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    match ams::initialize(root) {
        Ok(state) => println!(
            "A Magic Sovereign — engine ready (seed {}, {} Glyphs, {} mods)",
            state.seed,
            state.glyph_registry.len(),
            state.loaded_mods.len()
        ),
        Err(error) => {
            eprintln!("A Magic Sovereign failed to start: {error}");
            std::process::exit(1);
        }
    }
}
