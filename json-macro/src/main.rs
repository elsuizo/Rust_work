/// Ejemplo de como crear un json object desde una macro

use std::collections::HashMap;

#[derive(PartialEq, Debug)]
enum Json {
   Null,
   Boolean(bool),
   Number(f64),
   String(String),
   Array(Vec<Json>),
   Object(Box<HashMap<String, Json>>)
}

macro_rules! json {
    (null) => {
        Json::Null
    }
}

fn main() {
    println!("Hello, world!");
}

#[test]
fn json_null() {
    assert_eq!(json!(null), Json::Null);
}
