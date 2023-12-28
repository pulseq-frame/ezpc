// We test ezpc by writing a standards compliant json parser and running it on a
// json test suite, especially checking if the error messages are acceptable.

mod json_parser;
use std::fs;

use self::json_parser::json;

fn init() {
    let _ = env_logger::builder().is_test(false).try_init();
}

#[test]
/// Parse all files that should succeed
fn test_suite_y() {
    let paths = fs::read_dir("src/tests/JSONTestSuite/test_parsing").unwrap();

    for path in paths {
        let path = path.unwrap();
        let name = path.file_name().to_str().unwrap().to_owned();
        if name.starts_with("y_") {
            println!("{name}");
            let source = fs::read_to_string(path.path()).unwrap();
            assert!(
                json().parse_all(&source).is_ok(),
                "Failed to parse '{name}'"
            );
        }
    }
}

#[test]
/// Parse all files that should succeed
fn test_suite_n() {
    let paths = fs::read_dir("src/tests/JSONTestSuite/test_parsing").unwrap();

    for path in paths {
        let path = path.unwrap();
        let name = path.file_name().to_str().unwrap().to_owned();
        if name.starts_with("n_") {
            let source = fs::read(path.path()).unwrap();
            // Some files contain invalid utf8, which are ignored
            if let Ok(source) = std::str::from_utf8(&source) {
                println!("{name}");
                assert!(
                    json().parse_all(&source).is_err(),
                    "Parsed despite having errors '{name}'"
                );
            }
        }
    }
}

#[test]
/// Parse files which might or might not succeed - but parser should not crash
fn test_suite_i() {
    let paths = fs::read_dir("src/tests/JSONTestSuite/test_parsing").unwrap();

    for path in paths {
        let path = path.unwrap();
        let name = path.file_name().to_str().unwrap().to_owned();
        if name.starts_with("i_") {
            let source = fs::read(path.path()).unwrap();
            // Some files contain invalid utf8, which are ignored
            if let Ok(source) = std::str::from_utf8(&source) {
                println!("{name}");
                println!("{:?}", json().parse_all(&source));
            }
        }
    }
}

// TODO: all files that throw an error should have good error messages
// #[test]
// fn test_single_file() {
//     init();
//     let source = include_str!("JSONTestSuite/test_parsing/n_structure_100000_opening_arrays.json");
//     json().parse_all(source).unwrap();
// }
