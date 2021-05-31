use rusqlite_macros::validate;

#[test]
fn test_literal() {
    let sql = validate!("SELECT 1");
    assert_eq!(sql, "SELECT 1");
}

#[test]
fn test_raw_string() {
    let sql = validate!(r#"SELECT 1"#);
    assert_eq!(sql, "SELECT 1");
}

#[test]
fn test_const() {
    const SQL: &str = "SELECT 1";
    let sql = validate!(SQL);
    assert_eq!(sql, SQL);
}

#[test]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/*.rs");
}
