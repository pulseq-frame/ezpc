// We test ezpc by writing a standards compliant json parser and running it on a
// json test suite, especially checking if the error messages are acceptable.

mod json_parser;
use std::fs;

use self::json_parser::json;
use std::io::Write;

#[test]
/// Print the automatically generated description of the json parser
fn print_parser() {
    println!("{}", json());
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
            match json().parse_all(&source) {
                Ok(_) => (),
                Err(err) => panic!("Failed to parse '{name}': {err}"),
            }
        }
    }
}

#[test]
/// Parse all files that should succeed
fn test_suite_n() {
    let paths = fs::read_dir("src/tests/JSONTestSuite/test_parsing").unwrap();
    let mut error_file = fs::File::create("src/tests/output.txt").unwrap();

    for path in paths {
        let path = path.unwrap();
        let name = path.file_name().to_str().unwrap().to_owned();
        if name.starts_with("n_") {
            let source = fs::read(path.path()).unwrap();
            // Some files contain invalid utf8, which are ignored
            if let Ok(source) = std::str::from_utf8(&source) {
                println!("{name}");
                match json().parse_all(&source) {
                    Ok(_) => panic!("Parsed despite having errors '{name}'"),
                    Err(err) => writeln!(error_file, "{name:64}- {err}").unwrap(),
                }
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
