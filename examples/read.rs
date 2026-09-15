use appcfg::{Config, ConfigDirectory};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct AppSettings {
    port: u16,
    verbose: bool,
}
impl Default for AppSettings {
    fn default() -> Self {
        Self {
            port: 9000,
            verbose: true,
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
