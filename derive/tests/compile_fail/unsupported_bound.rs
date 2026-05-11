use config_derive::Deserialize;

#[derive(Deserialize)]
#[serde(bound = "T: Clone")]
struct MyStruct<T> {
    field: T,
}

fn main() {}
