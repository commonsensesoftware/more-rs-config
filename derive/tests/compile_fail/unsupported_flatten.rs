use config_derive::Deserialize;

#[derive(Deserialize)]
struct MyStruct {
    #[serde(flatten)]
    inner: std::collections::HashMap<String, String>,
}

fn main() {}
