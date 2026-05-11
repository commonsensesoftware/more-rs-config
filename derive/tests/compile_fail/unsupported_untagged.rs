use config_derive::Deserialize;

#[derive(Deserialize)]
#[serde(untagged)]
struct MyStruct {
    field: String,
}

fn main() {}
