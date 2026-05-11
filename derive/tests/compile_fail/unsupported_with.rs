use config_derive::Deserialize;

#[derive(Deserialize)]
struct MyStruct {
    #[serde(with = "some_module")]
    field: String,
}

fn main() {}
