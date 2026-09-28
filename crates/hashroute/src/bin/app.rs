use anyhow::Result;
use hashroute::internal::platform::application::Application;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;

    let app = Application::default();

    app.start_server().await?;

    Ok(())
}
