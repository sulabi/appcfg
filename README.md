# configfs

A small lightweight filesystem config manager
`configfs` provides an api to load, deserialize and write
config files for your application.
This currently only supports the TOML format, however I shall
introduce more formats in the future.

## Usage

```rust
use configfs::{Config, ConfigDirectory};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct AppSettings {
    port: u16,
    verbose: bool
}
impl Default for AppSettings {
    fn default() -> Self {
        Self {
            port: 9000,
            verbose: true
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new(ConfigDirectory::Custom("./appconf".into()))?;
    let settings = config.read_or_default::<AppSettings>()?;

    if settings.verbose {
        println!("using port: {}", settings.port);
    }

    Ok(())
}
```

## Writing Config

```rust
use configfs::{Config, ConfigDirectory};
use serde::Serialize;

#[derive(Serialize)]
struct AppSettings {
    username: String
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new(ConfigDirectory::Custom("./appconf".into()))?;
    let settings = AppSettings {
        username: "jimmy".into()
    };

    config.write(&settings)?;

    Ok(())
}
```
