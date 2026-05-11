use config_derive::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "type")]
struct MyStruct {
    field: String,
}

fn main() {}
