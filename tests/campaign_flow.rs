use ams::{
    app::{App, AppAction},
    glyphs::Element,
    saves::{read_save, SAVE_FORMAT_VERSION},
};
use std::{fs, path::Path};

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn game() -> (tempfile::TempDir, App) {
    let root = tempfile::tempdir().unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR"));
    fs::copy(source.join("config.toml"), root.path().join("config.toml")).unwrap();
    copy_tree(&source.join("data"), &root.path().join("data"));
    copy_tree(&source.join("gamemodes"), &root.path().join("gamemodes"));
    let app = App::new(ams::initialize(root.path()).unwrap());
    (root, app)
}

fn create(app: &mut App, elements: Vec<Element>) -> String {
    app.dispatch(AppAction::NewCharacter {
        save_name: "save-01".into(),
        character_name: "Éowyn".into(),
        race_id: "base:elf".into(),
        elements,
        gamemode_id: "base:standard".into(),
    })
}

#[test]
fn creates_unicode_character_with_zero_or_two_elements_and_persists_campaign() {
    let (root, mut app) = game();
    assert_eq!(app.state.race_registry.len(), 6);
    let response = create(&mut app, vec![Element::Fire, Element::Water]);
    assert!(response.contains("criado e salvo"), "{response}");
    assert_eq!(app.player.as_ref().unwrap().name, "Éowyn");
    assert_eq!(app.player.as_ref().unwrap().hp.maximum, 100);
    assert_eq!(app.player.as_ref().unwrap().tenacity.maximum, 50);
    assert_eq!(app.player.as_ref().unwrap().mana.maximum, 30);
    assert_eq!(
        app.player
            .as_ref()
            .unwrap()
            .attributes
            .get(ams::attributes::Attribute::Vigor),
        0
    );
    assert!(app.character.as_ref().unwrap().spells.is_empty());
    let path = root.path().join("saves/save-01.zip");
    let save = read_save(&path).unwrap();
    assert_eq!(save.metadata.save_format_version, SAVE_FORMAT_VERSION);
    assert_eq!(save.metadata.save_id, "save-01");
    let character = save.character.unwrap();
    assert_eq!(character.race_id, "base:elf");
    assert_eq!(character.elements.len(), 2);
    assert_eq!(
        save.campaign.unwrap().current_scene_id.as_deref(),
        Some("core:prologue_work_exit")
    );

    let mut empty = App::new(ams::initialize(root.path()).unwrap());
    let response = empty.dispatch(AppAction::NewCharacter {
        save_name: "empty-affinity".into(),
        character_name: "A".into(),
        race_id: "base:human".into(),
        elements: vec![],
        gamemode_id: "base:standard".into(),
    });
    assert!(response.contains("criado e salvo"));
    let saved = read_save(&root.path().join("saves/empty-affinity.zip")).unwrap();
    assert!(saved.character.unwrap().elements.is_empty());
}

#[test]
fn rejects_invalid_character_and_third_element_before_creating_a_save() {
    let (root, mut app) = game();
    assert!(create(
        &mut app,
        vec![Element::Fire, Element::Water, Element::Earth]
    )
    .contains("no máximo dois"));
    assert!(app.player.is_none());
    assert!(!root.path().join("saves/save-01.zip").exists());
    let result = app.dispatch(AppAction::NewCharacter {
        save_name: "invalid".into(),
        character_name: "".into(),
        race_id: "base:elf".into(),
        elements: vec![],
        gamemode_id: "base:standard".into(),
    });
    assert!(result.contains("1 a 64"));
}

#[test]
fn skip_and_scene_sequence_share_the_same_day_seven_transition_and_validate_travel() {
    let (root, mut app) = game();
    assert!(create(&mut app, vec![]).contains("criado e salvo"));
    for _ in 0..7 {
        app.dispatch(AppAction::AdvanceScene);
    }
    assert_eq!(
        app.campaign.as_ref().unwrap().current_scene_id.as_deref(),
        Some("core:prologue_seven_days")
    );
    let mut resumed = App::new(ams::initialize(root.path()).unwrap());
    let loaded = resumed.dispatch(AppAction::RequestLoad {
        save_name: "save-01".into(),
    });
    assert!(loaded.contains("Loaded"), "{loaded}");
    assert_eq!(
        resumed
            .campaign
            .as_ref()
            .unwrap()
            .current_scene_id
            .as_deref(),
        Some("core:prologue_seven_days")
    );
    let result = resumed.dispatch(AppAction::AdvanceScene);
    assert!(result.contains("Prólogo concluído"));
    let campaign = resumed.campaign.as_ref().unwrap();
    assert!(campaign.prologue_completed);
    assert_eq!(campaign.day, 7);
    assert_eq!(campaign.location_id.as_deref(), Some("core:tidal_town"));
    assert!(resumed
        .dispatch(AppAction::MoveToLocation {
            location_id: "core:missing".into()
        })
        .contains("cannot travel"));
    assert!(resumed
        .dispatch(AppAction::MoveToLocation {
            location_id: "core:tidal_town_square".into()
        })
        .contains("Praça"));
    assert_eq!(
        resumed.campaign.as_ref().unwrap().location_id.as_deref(),
        Some("core:tidal_town_square")
    );
    assert!(resumed.dispatch(AppAction::Save).contains("Saved"));
    let mut after_travel = App::new(ams::initialize(root.path()).unwrap());
    assert!(after_travel
        .dispatch(AppAction::RequestLoad {
            save_name: "save-01".into()
        })
        .contains("Loaded"));
    assert_eq!(
        after_travel
            .campaign
            .as_ref()
            .unwrap()
            .location_id
            .as_deref(),
        Some("core:tidal_town_square")
    );

    let mut skipped = App::new(ams::initialize(root.path()).unwrap());
    let created = skipped.dispatch(AppAction::NewCharacter {
        save_name: "skip-save".into(),
        character_name: "Skip".into(),
        race_id: "base:human".into(),
        elements: vec![],
        gamemode_id: "base:standard".into(),
    });
    assert!(created.contains("criado e salvo"));
    let skipped_result = skipped.dispatch(AppAction::SkipPrologue);
    assert!(skipped_result.contains("dia 7"));
    assert_eq!(
        skipped.campaign.as_ref().unwrap().location_id.as_deref(),
        Some("core:tidal_town")
    );
}

#[test]
fn missing_scene_is_reported_without_advancing_or_corrupting_campaign() {
    let (_root, mut app) = game();
    assert!(create(&mut app, vec![]).contains("criado e salvo"));
    app.state.scene_registry = ams::campaign::SceneRegistry::default();
    let response = app.dispatch(AppAction::AdvanceScene);
    assert!(response.contains("Cena ausente ou inválida"));
    assert_eq!(
        app.campaign.as_ref().unwrap().current_scene_id.as_deref(),
        Some("core:prologue_work_exit")
    );
}
