use tokio::sync::OnceCell;

#[derive(Debug)]
pub struct ClientConfig {
    pub cert_file: String,
    pub addr: String,
    pub port: u16,
    pub domain: String
}

static CONFIG: OnceCell<ClientConfig> = OnceCell::const_new();

pub async fn init_config(config: ClientConfig) {
    CONFIG.set(config).unwrap();
}

pub async fn get_config() -> &'static ClientConfig {
    CONFIG.get().unwrap()
}
