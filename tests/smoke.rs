#[test]
fn test_crate_import_smoke() {
    let result = bash_classify::classify_expression("ls", None).unwrap();
    assert_eq!(
        result.classification,
        bash_classify::Classification::Readonly
    );
}
