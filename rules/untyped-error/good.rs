enum FetchError {
    Auth { refresh_token: RefreshToken },
    Network { retry_after: Duration },
    RateLimited { retry_after: Duration },
    Validation(ValidationError),
}

fn fetch(url: &Url) -> Result<Response, FetchError> {
    http::get(url).map_err(FetchError::from)
}

impl Client {
    fn fetch_with_retry(&self, url: &Url) -> Result<Response, FetchError> {
        match fetch(url) {
            Ok(response) => Ok(response),
            Err(FetchError::Network { retry_after } | FetchError::RateLimited { retry_after }) => {
                self.clock.sleep(retry_after);
                fetch(url)
            }
            Err(FetchError::Auth { refresh_token }) => {
                self.refresh(refresh_token)?;
                fetch(url)
            }
            Err(error @ FetchError::Validation(_)) => Err(error),
        }
    }
}
