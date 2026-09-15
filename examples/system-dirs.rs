use configfs::{Config, ConfigDirectory};
use serde::Serialize;

#[derive(Serialize)]
struct AppSettings {
    username: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new(ConfigDirectory::System("app"))?;
    let settings = AppSettings {
        username: "jimmy".into(),
    };

    config.write(&settings)?;

    Ok(())
}
