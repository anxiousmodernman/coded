extern crate coded;
use std::path::PathBuf;
use coded::*;

#[test]
fn fileinfo_from_path() {
    // path, expected lines, expected extension
    let cases = vec![
        ("tests/data/main.go", 8, "go"),
        ("tests/data/pkg.go", 1, "go"),
        ("tests/data/data.txt", 1, "txt"),
    ];

    for case in cases {
        let path = PathBuf::from(case.0);
        let fi = project::FileInfo::from_path(&path).unwrap();
        assert_eq!(fi.lines, case.1);
        assert_eq!(fi.extension, case.2);
    }
}