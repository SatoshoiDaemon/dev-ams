use ams::{
    attributes::{AttributeRules, AttributeSet},
    content::StableId,
    entities::{Allegiance, Entity},
    lua::{create_runtime, MOD_API_VERSION},
    saves::{read_save, write_save, SaveData, SaveMetadata, SAVE_FORMAT_VERSION},
};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn save_round_trip_preserves_human_readable_opaque_mod_data() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("character.zip");
    let entity = Entity::new(
        StableId::new("base:hero").unwrap(),
        "Hero",
        Allegiance::Player,
        AttributeSet::default(),
        &AttributeRules::default(),
    )
    .unwrap();
    let mut opaque = BTreeMap::new();
    opaque.insert(
        "author:example".into(),
        json!({"unknown_future_field": [1, 2, 3]}),
    );
    let original = SaveData {
        metadata: SaveMetadata {
            save_format_version: SAVE_FORMAT_VERSION,
            ruleset_version: "1".into(),
            required_mods: vec![],
        },
        entities: vec![entity],
        opaque_mod_data: opaque,
    };
    write_save(&path, &original).unwrap();
    assert_eq!(read_save(&path).unwrap(), original);
}

#[test]
fn lua_runtime_is_embedded_versioned_and_has_no_os_access() {
    let lua = create_runtime().unwrap();
    assert_eq!(
        lua.globals().get::<u32>("mod_api_version").unwrap(),
        MOD_API_VERSION
    );
    assert!(lua.globals().get::<mlua::Value>("io").unwrap().is_nil());
    assert!(lua.globals().get::<mlua::Value>("os").unwrap().is_nil());
    assert!(lua
        .globals()
        .get::<mlua::Value>("package")
        .unwrap()
        .is_nil());
}
