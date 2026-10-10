use ams::{
    attributes::{AttributeRules, AttributeSet},
    content::StableId,
    entities::{Allegiance, Entity},
    lua::{create_runtime, MOD_API_VERSION},
    saves::{read_save, write_save, SaveData, SaveMetadata, SAVE_FORMAT_VERSION},
};
use serde_json::json;
use std::{collections::BTreeMap, fs::File, io::Write};
use zip::{write::SimpleFileOptions, ZipWriter};

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
            save_id: "character".into(),
            ruleset_version: "1".into(),
            required_mods: vec![],
            gamemode_id: "base:standard".into(),
            player_entity_ids: vec!["base:hero".into()],
            rng_state: 1,
        },
        ruleset: json!({"ruleset_version": "1"}),
        entities: vec![entity],
        character: None,
        campaign: None,
        combat: None,
        opaque_mod_data: opaque,
        migration_warnings: Vec::new(),
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

#[test]
fn v1_save_migrates_without_combat_and_reports_missing_snapshot() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("legacy.zip");
    let mut archive = ZipWriter::new(File::create(&path).unwrap());
    let options = SimpleFileOptions::default();
    archive.start_file("metadata.json", options).unwrap();
    archive
        .write_all(br#"{"save_format_version":1,"ruleset_version":"base:standard@1"}"#)
        .unwrap();
    archive.start_file("entities.json", options).unwrap();
    archive.write_all(b"[]").unwrap();
    archive.start_file("mod_data.json", options).unwrap();
    archive
        .write_all(br#"{"missing:legacy":{"kept":true}}"#)
        .unwrap();
    archive.finish().unwrap();

    let migrated = read_save(&path).unwrap();
    assert_eq!(migrated.metadata.save_format_version, SAVE_FORMAT_VERSION);
    assert!(migrated.combat.is_none());
    assert_eq!(migrated.opaque_mod_data["missing:legacy"]["kept"], true);
    assert_eq!(migrated.migration_warnings.len(), 1);
}

#[test]
fn v2_demo_save_migrates_without_inventing_campaign_state() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("old-demo.zip");
    let mut archive = ZipWriter::new(File::create(&path).unwrap());
    let options = SimpleFileOptions::default();
    for (name, contents) in [
        ("metadata.json", br#"{"save_format_version":2,"ruleset_version":"base:standard@1","gamemode_id":"base:standard","rng_state":17}"#.as_slice()),
        ("ruleset.json", br#"{"ruleset_version":"base:standard@1"}"#.as_slice()),
        ("entities.json", b"[]".as_slice()),
        ("mod_data.json", b"{}".as_slice()),
    ] {
        archive.start_file(name, options).unwrap();
        archive.write_all(contents).unwrap();
    }
    archive.finish().unwrap();
    let migrated = read_save(&path).unwrap();
    assert_eq!(migrated.metadata.save_format_version, SAVE_FORMAT_VERSION);
    assert_eq!(migrated.metadata.save_id, "old-demo");
    assert!(migrated.character.is_none());
    assert!(migrated.campaign.is_none());
    assert!(migrated.combat.is_none());
}

#[test]
fn replacing_a_save_keeps_the_zip_readable_and_uses_the_new_contents() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("replace.zip");
    let make = |save_id: &str| SaveData {
        metadata: SaveMetadata {
            save_format_version: SAVE_FORMAT_VERSION,
            save_id: save_id.into(),
            ruleset_version: "base:standard@1".into(),
            required_mods: vec![],
            gamemode_id: "base:standard".into(),
            player_entity_ids: vec![],
            rng_state: 1,
        },
        ruleset: json!({}),
        entities: vec![],
        character: None,
        campaign: None,
        combat: None,
        opaque_mod_data: BTreeMap::new(),
        migration_warnings: vec![],
    };
    write_save(&path, &make("first")).unwrap();
    write_save(&path, &make("second")).unwrap();
    assert_eq!(read_save(&path).unwrap().metadata.save_id, "second");
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
}
