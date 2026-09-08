async fn start_server(app: App) -> Result<(), ServerError> {
    let port: u16 = std::env::var("PORT")?.parse()?;
    app.listen(port).await
}
