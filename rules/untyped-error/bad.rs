fn fetch(url: &Url) -> Result<Response, Box<dyn Error>> {
    Ok(http::get(url)?)
}

fn parse(input: &str) -> Result<Config, String> {
    toml::from_str(input).map_err(|error| error.to_string())
}
