use wizard_blog_backend::{
    configuration::get_configuration,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Set logging output to a file
    // let ts = Local::now().format("%Y-%m-%d_%H-%M-%S");
    // let log_file = format!("log_{}.txt", ts);
    // let file = File::create(&log_file)?;
    let subscriber = get_subscriber("wizard-blog-backend".into(), "info".into(), std::io::stdout);

    init_subscriber(subscriber);

    let configuration = get_configuration().expect("failed to read configuration.");

    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}
