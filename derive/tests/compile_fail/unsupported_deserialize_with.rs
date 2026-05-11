use config_derive::Deserialize;

#[derive(Deserialize)]
struct MyStruct {
    #[serde(deserialize_with = "some_function")]
    field: String,
}

fn main() {}
