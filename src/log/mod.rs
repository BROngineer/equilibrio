use tracing::dispatcher::SetGlobalDefaultError;
use tracing::Level;

pub enum Format {
    PlainText,
    Json,
}

pub fn setup(level: Level, format: Format) -> Result<(), SetGlobalDefaultError> {
    match format {
        Format::PlainText => plain_text_subscriber(level),
        Format::Json => json_subscriber(level),
    }
}

fn json_subscriber(level: Level) -> Result<(), SetGlobalDefaultError> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(level)
        .with_line_number(false)
        .with_file(false)
        .with_target(true)
        .json()
        .finish();
    tracing::subscriber::set_global_default(subscriber)
}

fn plain_text_subscriber(level: Level) -> Result<(), SetGlobalDefaultError> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(level)
        .with_line_number(false)
        .with_file(false)
        .with_target(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
}
