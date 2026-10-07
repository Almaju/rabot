struct Config {
    db_url: DatabaseUrl,
    port: Port,
}

impl Config {
    fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            db_url: DatabaseUrl::parse(std::env::var("DATABASE_URL")?)?,
            port: Port::parse(std::env::var("PORT")?)?,
        })
    }
}

async fn start_server(app: App, port: Port) -> Result<(), ServerError> {
    app.listen(port).await
}

fn main() {
    let config = Config::from_env().expect("configuration required for startup");
    start_server(App::new(), config.port);
}
