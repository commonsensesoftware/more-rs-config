use config_derive::Deserialize;

#[derive(Deserialize)]
#[serde(try_from = "String")]
struct MyStruct {
    field: String,
}

fn main() {}
