use config::{prelude::*, Deserialize};
use std::{error::Error, path::Path};

#[derive(Default, Deserialize)]
struct Client {
    region: String,
    url: String,
}

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
    let mut app = AppOptions::default();
    let before = &mut app as *mut _;

    config.bind(&mut app)?;

    let after = &mut app as *mut _;

    // verify the pointer wasn't replaced
    assert_eq!(before, after, "app was replaced");

    if app.demo {
        println!("Text = {}", &app.text);
        println!("Region = {}", &app.clients[0].region);
        println!("URL = {}", &app.clients[0].url);
    } else {
        println!("Not a demo!");
    }

    Ok(())
}
