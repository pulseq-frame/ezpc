#[path = "../benches/json_ezpc.rs"]
mod json_ezpc;
use json_ezpc::json;

fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Trace).init();

    let source = std::fs::read_to_string("benches/data.json").unwrap();
    let parsed = json().parse_all(&source);
    println!("{parsed:?}");
}
