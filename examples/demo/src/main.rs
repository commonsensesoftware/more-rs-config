use config::{*, ext::*};
use serde::{Serialize, Deserialize};

#[allow(dead_code)]
#[derive(Default, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "PascalCase"))]
#[serde(rename_all(deserialize = "PascalCase"))]
struct Client {
    region: String,
    url: String,
}

#[allow(dead_code)]
#[derive(Default, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "PascalCase"))]
#[serde(rename_all(deserialize = "PascalCase"))]
struct SubOptions {
    value: i32,
}

#[allow(dead_code)]
#[derive(Default, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "PascalCase"))]
#[serde(rename_all(deserialize = "PascalCase"))]
struct AppOptions {
    text: String,
    demo: bool,
    sub: SubOptions,
    clients: Vec<Client>,
}

fn main() {
    let file = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("../../examples/demo/demo.json");

    let default = AppOptions {
        text: String::from("Default text"),
        demo: false,
        sub: SubOptions { value: 34},
        clients: Vec::new(),
    };

    let config = DefaultConfigurationBuilder::new()
        .add_struct(default.clone())
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
        println!("Suboption value by query: {}", config.get("Sub:Value").unwrap().as_str());
        println!("Suboption value by binding: {}", &app.sub.value);
        return;
    }
    
    println!("Not a demo!");
}
