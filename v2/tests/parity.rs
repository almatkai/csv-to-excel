use std::{fs, path::Path};
use tezbar_csv_to_excel::{
    convert, parse, workbook, MAX_CELLS, MAX_COLUMNS, MAX_INPUT_BYTES, MAX_ROWS,
};
#[test]
fn exact_source_corpus() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/golden");
    let mut count = 0;
    for entry in fs::read_dir(root).unwrap() {
        let input = entry.unwrap().path();
        if input.extension().and_then(|s| s.to_str()) != Some("csv") {
            continue;
        }
        let actual = convert(&fs::read(&input).unwrap());
        let error = input.with_extension("error");
        if error.exists() {
            assert_eq!(
                actual.unwrap_err(),
                fs::read_to_string(error).unwrap(),
                "{}",
                input.display()
            );
        } else {
            assert_eq!(
                actual.unwrap(),
                fs::read(input.with_extension("xlsx")).unwrap(),
                "{}",
                input.display()
            );
        }
        count += 1;
    }
    assert!(count == 288, "golden corpus missing cases");
}
#[test]
fn explicit_limits_never_truncate() {
    assert!(convert(&vec![b'x'; MAX_INPUT_BYTES + 1])
        .unwrap_err()
        .contains("resource limit"));
    assert!(parse(&",".repeat(MAX_COLUMNS))
        .unwrap_err()
        .contains("resource limit"));
    assert!(parse(&"a\n".repeat(MAX_ROWS + 1))
        .unwrap_err()
        .contains("resource limit"));
    let row = vec![String::from("x"); 2500];
    let rows = vec![row; MAX_CELLS / 2500 + 1];
    assert!(workbook(&rows).unwrap_err().contains("resource limit"));
    assert_eq!(
        convert(format!("a\n{}", "x".repeat(32768)).as_bytes()).unwrap_err(),
        "Text length must not exceed 32767 characters"
    );
}

#[test]
fn output_limit_is_an_error_before_publication() {
    let header = (0..700).map(|i| format!("h{i}")).collect();
    let row = vec!["&".repeat(20_000); 700];
    assert!(workbook(&[header, row])
        .unwrap_err()
        .contains("64 MiB resource limit"));
}
