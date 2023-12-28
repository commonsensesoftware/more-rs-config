use config::{*, ext::*};
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Default, Deserialize)]
#[serde(rename_all(deserialize = "PascalCase"))]
struct Client {
    region: String,
    url: String,
}

#[allow(dead_code)]
#[derive(Default, Deserialize)]
#[serde(rename_all(deserialize = "PascalCase"))]
struct AppOptions {
    text: String,
    demo: bool,
    clients: Vec<Client>,
}

fn main() {
    let file = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("../../examples/demo/demo.json");
    let config = DefaultConfigurationBuilder::new()
        .add_in_memory(&[("Demo", "false")])
        .add_json_file(file)
        .add_env_vars()
        .add_command_line()
        .build()
        .unwrap();
    let app: AppOptions = config.reify();
    
    if app.demo {
        println!("{}", &app.text);
        println!("{}", &app.clients[0].region);
        return;
    }
    
    println!("Not a demo!");
}