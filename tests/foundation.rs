use std::fs;
use ams::{combat::{damage::split_tenacity_damage, DeterministicRng}, config::AppConfig, content::{load_json_registry, StableId}};
#[test] fn config_defaults_are_portable() { let config = AppConfig::default(); assert_eq!(config.paths.saves, "saves"); assert_eq!(config.numeric.power_per_attribute, 7); }
#[test] fn stable_ids_require_namespaces() { assert!(StableId::new("base:goblin").is_ok()); assert!(StableId::new("goblin").is_err()); }
#[test] fn rng_is_reproducible() { let mut a = DeterministicRng::new(7); let mut b = DeterministicRng::new(7); assert_eq!(a.next_u64(), b.next_u64()); assert_eq!(a.next_u64(), b.next_u64()); }
#[test] fn tenacity_split_is_deterministic() { assert_eq!(split_tenacity_damage(5), (2, 3)); }
#[test] fn content_load_order_is_stable() { let dir = tempfile::tempdir().unwrap(); fs::write(dir.path().join("z.json"), r#"{"id":"base:z","name":"Z"}"#).unwrap(); fs::write(dir.path().join("a.json"), r#"{"id":"base:a","name":"A"}"#).unwrap(); let registry = load_json_registry(dir.path()).unwrap(); let ids: Vec<_> = registry.iter().map(|(id, _)| id.as_str()).collect(); assert_eq!(ids, vec!["base:a", "base:z"]); }
