use ams::modding::discover_mods_report;
use std::fs;

fn manifest(id: &str, dependencies: &[&str]) -> String {
    serde_json::json!({
        "id": id,
        "name": id,
        "version": "1.0.0",
        "api_version": 1,
        "dependencies": dependencies,
    })
    .to_string()
}

#[test]
fn invalid_mod_is_isolated_and_dependents_are_blocked() {
    let directory = tempfile::tempdir().unwrap();
    let valid = directory.path().join("a-valid");
    let invalid = directory.path().join("b-invalid");
    let dependent = directory.path().join("c-dependent");
    fs::create_dir(&valid).unwrap();
    fs::create_dir(&invalid).unwrap();
    fs::create_dir(&dependent).unwrap();
    fs::write(valid.join("manifest.json"), manifest("test:valid", &[])).unwrap();
    fs::write(invalid.join("manifest.json"), "{not-json").unwrap();
    fs::write(
        dependent.join("manifest.json"),
        manifest("test:dependent", &["test:missing"]),
    )
    .unwrap();

    let report = discover_mods_report(directory.path()).unwrap();
    assert_eq!(report.loaded.len(), 1);
    assert_eq!(report.loaded[0].manifest.id.as_str(), "test:valid");
    assert_eq!(report.failed.len(), 2);
    assert!(report
        .failed
        .iter()
        .any(|failure| failure.stage == "manifest"));
    assert!(report
        .failed
        .iter()
        .any(|failure| failure.stage == "dependency resolution"));
}
