use anyhow::Result;
use rast::settings::Settings;
use rast_agent::RastAgent;
use tracing::info;
use tracing_subscriber::filter::LevelFilter;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();

    // TODO: add embedding during compile
    let settings = match Settings::new() {
        Ok(settings) => {
            info!("{settings:?}");
            settings
        },
        Err(e) => {
            panic!("{e:?}");
        },
    };

    let mut agent = RastAgent::with_settings(settings).await?;
    agent.run().await?;

    Ok(())
}
