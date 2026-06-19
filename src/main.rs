use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use dnsmasqweb::server::state::AppState;
use dnsmasqweb::server::{auth, routes};
use tokio::net::TcpListener;
use tracing::info;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    #[arg(long, env = "DNSMASQWEB_CONFIG", default_value = "/etc/dnsmasq.conf")]
    config: PathBuf,

    #[arg(
        long,
        env = "DNSMASQWEB_BACKUP_DIR",
        default_value = "/var/backups/dnsmasqweb"
    )]
    backup_dir: PathBuf,

    #[arg(long, env = "DNSMASQWEB_LISTEN", default_value = "127.0.0.1:8080")]
    listen: SocketAddr,

    #[arg(
        long,
        env = "DNSMASQWEB_DNSMASQ_BIN",
        default_value = "/usr/sbin/dnsmasq"
    )]
    dnsmasq_bin: String,

    #[arg(long, env = "DNSMASQWEB_SERVICE", default_value = "dnsmasq")]
    service: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    let state = AppState::new(cli.config, cli.backup_dir, cli.dnsmasq_bin, cli.service);
    tokio::spawn(auth::cleanup_expired_sessions(state.clone()));

    let app = routes::router(state);
    let listener = TcpListener::bind(cli.listen)
        .await
        .with_context(|| format!("bind {}", cli.listen))?;

    info!("listening on http://{}", cli.listen);
    axum::serve(listener, app).await?;
    Ok(())
}
