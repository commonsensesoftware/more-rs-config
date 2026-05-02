use config::prelude::*;
use serde::Deserialize;
use std::{error::Error, path::Path};

#[allow(dead_code)]
#[derive(Default, Deserialize)]
struct Client {
    region: String,
    url: String,
}

#[allow(dead_code)]
#[derive(Default, Deserialize)]
struct AppOptions {
    text: String,
    demo: bool,
    clients: Vec<Client>,
}

fn main() -> Result<(), Box<dyn Error + 'static>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples").join("demo.json");
    let config = config::builder()
        .add_in_memory(&[("Demo", "false")])
        .add_json_file(path)
        .add_env_vars()
        .add_command_line()
        .build()?;
    let app: AppOptions = config.reify()?;

    if app.demo {
        println!("Text = {}", &app.text);
        println!("Region = {}", &app.clients[0].region);
    } else {
        println!("Not a demo!");
    }

    Ok(())
}
