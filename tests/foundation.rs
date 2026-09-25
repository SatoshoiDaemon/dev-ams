use ams::{
    combat::{damage::split_tenacity_damage, DeterministicRng},
    config::AppConfig,
    content::{load_json_registry, StableId},
};
use std::fs;
#[test]
fn config_defaults_are_portable() {
    let config = AppConfig::default();
    assert_eq!(config.paths.saves, "saves");
    assert_eq!(config.numeric.power_per_attribute, 7);
}
#[test]
fn stable_ids_require_namespaces() {
    assert!(StableId::new("base:goblin").is_ok());
    assert!(StableId::new("goblin").is_err());
}
#[test]
fn rng_is_reproducible() {
    let mut a = DeterministicRng::new(7);
    let mut b = DeterministicRng::new(7);
    assert_eq!(a.next_u64(), b.next_u64());
    assert_eq!(a.next_u64(), b.next_u64());
}
#[test]
fn tenacity_split_is_deterministic() {
    assert_eq!(split_tenacity_damage(5), (3, 2));
}
#[test]
fn content_load_order_is_stable() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("z.json"), r#"{"id":"base:z","name":"Z"}"#).unwrap();
    fs::write(dir.path().join("a.json"), r#"{"id":"base:a","name":"A"}"#).unwrap();
    let registry = load_json_registry(dir.path()).unwrap();
    let ids: Vec<_> = registry.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(ids, vec!["base:a", "base:z"]);
}

#[test]
fn gamemode_overrides_global_numeric_values_by_internal_id() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(
        directory.path().join("config.toml"),
        "[numeric]\nhp_per_vigor = 12\nmp_per_mana = 11\n",
    )
    .unwrap();
    let modes = directory.path().join("gamemodes");
    fs::create_dir(&modes).unwrap();
    fs::write(
        modes.join("filename-does-not-define-id.toml"),
        "id = \"test:mode\"\nruleset_version = \"test:mode@1\"\n[numeric]\nhp_per_vigor = 20\n",
    )
    .unwrap();
    let config = ams::config::AppConfig::load(directory.path()).unwrap();
    let registry = ams::config::GameModeRegistry::load(&modes).unwrap();
    let resolved = registry
        .get("test:mode")
        .unwrap()
        .resolve_numeric(&config.numeric);
    assert_eq!(resolved.hp_per_vigor, 20);
    assert_eq!(resolved.mp_per_mana, 11);
}
