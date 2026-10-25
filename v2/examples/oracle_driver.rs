use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let input: serde_json::Value = serde_json::from_str(&line).unwrap();
        let bytes: Vec<u8> = serde_json::from_value(input["bytes"].clone()).unwrap();
        let result = match tezbar_csv_to_excel::convert(&bytes) {
            Ok(bytes) => serde_json::json!({"bytes":bytes}),
            Err(message) => serde_json::json!({"error":message}),
        };
        println!("{result}");
    }
}
