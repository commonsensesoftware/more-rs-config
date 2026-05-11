use config_derive::Deserialize;

#[derive(Deserialize)]
#[serde(from = "String")]
struct MyStruct {
    field: String,
}

fn main() {}
