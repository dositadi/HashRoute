use anyhow::Result;
use hashroute::internal::platform::app::application::Application;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;

    let app = Application::default();

    app.start_server().await?;

    Ok(())
}
