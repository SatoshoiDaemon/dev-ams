use ams::{
    app::{App, AppAction, AppMode},
    saves::validate_save_name,
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

fn app_root() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR"));
    fs::copy(source.join("config.toml"), root.path().join("config.toml")).unwrap();
    copy_tree(&source.join("data"), &root.path().join("data"));
    copy_tree(&source.join("gamemodes"), &root.path().join("gamemodes"));
    root
}

#[test]
fn structured_actions_create_save_start_and_reload_unicode_save() {
    let root = app_root();
    let mut app = App::new(ams::initialize(root.path()).unwrap());
    assert!(app
        .dispatch(AppAction::NewGame {
            save_name: "Dragão Negro".into(),
            gamemode_id: "base:standard".into(),
        })
        .contains("Created"));
    assert!(app
        .dispatch(AppAction::StartCombat {
            encounter_id: "base:training-encounter".into(),
        })
        .contains("Combat started"));
    assert_eq!(app.mode, AppMode::Combat);
    assert!(!app.dispatch(AppAction::ListActions).is_empty());
    let response = app.dispatch(AppAction::UseAction {
        action_id: "base:unarmed-strike".into(),
        target_ids: vec!["base:raider-1".into()],
    });
    assert!(!response.contains("rejected"), "{response}");
    let _ = app.dispatch(AppAction::EndRound);
    let explanation = app.dispatch(AppAction::ExplainLast);
    assert!(
        !explanation.contains("No completed action"),
        "{explanation}"
    );
    assert!(app.dispatch(AppAction::Save).contains("Saved"));
    assert!(root.path().join("saves/Dragão Negro.zip").is_file());

    let mut reloaded = App::new(ams::initialize(root.path()).unwrap());
    assert!(reloaded
        .dispatch(AppAction::RequestLoad {
            save_name: "Dragão Negro".into(),
        })
        .contains("Loaded save"));
    assert_eq!(reloaded.mode, AppMode::Combat);
    assert!(!reloaded
        .dispatch(AppAction::ExplainLast)
        .contains("No completed action"));
    let _ = reloaded.dispatch(AppAction::EndRound);
}

#[test]
fn cli_parser_routes_through_actions_and_new_save_never_overwrites() {
    let root = app_root();
    let mut app = App::new(ams::initialize(root.path()).unwrap());
    assert!(validate_save_name("Éowyn").is_ok());
    assert!(app.command("new Éowyn").contains("Created"));
    assert!(app
        .command("combat base:training-encounter")
        .contains("Combat started"));
    assert!(app.command("save").contains("Saved"));

    let mut second = App::new(ams::initialize(root.path()).unwrap());
    assert!(second
        .dispatch(AppAction::NewGame {
            save_name: "Éowyn".into(),
            gamemode_id: "base:standard".into(),
        })
        .contains("already exists"));
    assert!(second
        .dispatch(AppAction::NewGame {
            save_name: "éOWYN".into(),
            gamemode_id: "base:standard".into(),
        })
        .contains("already exists"));
    assert!(second
        .command("new ../escape")
        .contains("invalid save name"));

    let mut unsaved = App::new(ams::initialize(root.path()).unwrap());
    unsaved.dispatch(AppAction::NewGame {
        save_name: "unsaved".into(),
        gamemode_id: "base:standard".into(),
    });
    unsaved.dispatch(AppAction::RequestQuit);
    unsaved.dispatch(AppAction::ConfirmQuit);
    assert!(unsaved.should_exit);
}
