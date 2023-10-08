use std::error::Error;
use std::fs;
use std::path::Path;
use directories::BaseDirs;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use language_server::start_lsp_server;

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let log_dir =
        if let Some(base_dirs) = BaseDirs::new() {
            base_dirs.data_dir().join("mongold").join("logs")
        } else {
            Path::new("/tmp/mongold/logs").to_path_buf()
        };

    fs::create_dir_all(&log_dir).expect("Could not create log directory.");

    let file_appender = tracing_appender::rolling::daily(log_dir.as_path(), "mongold.log");
    let json_format = tracing_subscriber::fmt::layer()
        .json().flatten_event(true)
        .with_writer(file_appender);

    tracing_subscriber::registry().with(json_format).init();

    info!(
        version = build_info::format!("{}", $.crate_info.version),
        compiler = build_info::format!("{}", $.compiler),
        target = build_info::format!("{}", $.target),
        profile = build_info::format!("{}", $.profile),
        git = build_info::format!("{}", $.version_control?.git()?.commit_id),
        "Starting mongold"
    );

    let io_threads = start_lsp_server()?;
    io_threads.join()?;

    info!("Stopping mongold gracefully.");
    return Ok(());
}
