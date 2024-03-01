use confique::Config as Configure;
use dotenv::dotenv;

#[derive(Configure, Clone)]
pub struct Config {
    #[config(env = "REDIS_HOST", default = "127.0.0.1")]
    pub redis_host: String,

    #[config(env = "REDIS_PORT", default = 6379)]
    pub redis_port: u16,

    #[config(env = "REDIS_USERNAME")]
    pub redis_user: Option<String>,

    #[config(env = "REDIS_PASSWORD")]
    pub redis_pass: Option<String>,

    #[config(env = "REDIS_DATABASE")]
    pub redis_db: Option<String>,
}

impl Config {
    pub fn new() -> Self {
        dotenv().ok();

        Self::builder()
            .env()
            .file("toretsu.json")
            .file("toretsu.yaml")
            .file("toretsu.toml")
            .load()
            .expect("Config Not Found")
    }
}
