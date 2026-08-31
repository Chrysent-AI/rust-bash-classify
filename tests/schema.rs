use std::fs;
use std::path::Path;

#[test]
fn test_yaml_validates_against_schema() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let schema_path = root.join("vendor/schemas/command.schema.json");
    let commands_dir = root.join("vendor/commands");

    let schema_text = fs::read_to_string(&schema_path).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_text).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();

    let mut yaml_files: Vec<_> = fs::read_dir(&commands_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("yaml"))
        .collect();
    yaml_files.sort();

    assert_eq!(
        yaml_files.len(),
        166,
        "expected the M0 vendored command count"
    );

    for yaml_file in yaml_files {
        let yaml_text = fs::read_to_string(&yaml_file).unwrap();
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&yaml_text)
            .unwrap_or_else(|err| panic!("{} failed YAML parsing: {err}", yaml_file.display()));
        let json_value = serde_json::to_value(yaml_value).unwrap_or_else(|err| {
            panic!(
                "{} failed YAML-to-JSON conversion: {err}",
                yaml_file.display()
            )
        });

        if let Err(error) = validator.validate(&json_value) {
            panic!("{} failed schema validation: {error}", yaml_file.display());
        }
    }
}
