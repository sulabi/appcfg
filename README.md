# Config Watcher

Simple config reader / writer.

## Example

```rust
use config::{Config, ConfigDirectory};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct AppConfig {
    age: u8,
    server_name: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            age: 1,
            server_name: "app_1".to_string(),
        }
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let disk_config = Config::new(ConfigDirectory::Custom(PathBuf::from("./custom file")))?
        .with_file("app.toml");

    let shared = disk_config
        .load_shared_or_default::<AppConfig>()?
        .on_reload(|conf| println!("new age: {}", conf.age));

    let _watcher = shared.clone().spawn_watcher()?;

    println!("Watching for changes on app.toml");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(())
}
```
