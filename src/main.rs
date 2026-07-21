use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use dnsmasqweb::server::state::AppState;
use dnsmasqweb::server::{auth, routes};
use leptos::config::{LeptosOptions, get_configuration};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

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

    #[arg(
        long,
        env = "DNSMASQWEB_CREDENTIALS_FILE",
        default_value = "/var/lib/dnsmasqweb/password.hash"
    )]
    credentials_file: PathBuf,

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
    init_tracing();
    let cli = Cli::parse();
    let leptos_options = load_leptos_options().context("load Leptos configuration")?;

    let state = AppState::new(
        leptos_options,
        cli.config,
        cli.backup_dir,
        cli.credentials_file,
        cli.dnsmasq_bin,
        cli.service,
    );
    auth::load_persisted_password(&state)
        .await
        .context("load persisted admin password")?;
    tokio::spawn(auth::cleanup_expired_sessions(state.clone()));

    let app = routes::router(state);
    let listener = TcpListener::bind(cli.listen)
        .await
        .with_context(|| format!("bind {}", cli.listen))?;

    info!("listening on http://{}", cli.listen);
    axum::serve(listener, app).await?;
    Ok(())
}

fn load_leptos_options() -> anyhow::Result<LeptosOptions> {
    if std::env::var_os("LEPTOS_OUTPUT_NAME").is_some() {
        return Ok(get_configuration(None)?.leptos_options);
    }

    let site_root = match std::env::var("LEPTOS_SITE_ROOT") {
        Ok(site_root) => site_root,
        Err(_) => default_site_root(),
    };

    let site_pkg_dir = std::env::var("LEPTOS_SITE_PKG_DIR").unwrap_or_else(|_| String::from("pkg"));

    Ok(LeptosOptions::builder()
        .output_name("dnsmasqweb")
        .site_root(site_root)
        .site_pkg_dir(site_pkg_dir)
        .build())
}

fn default_site_root() -> String {
    let release_package_site = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join("site")));

    if let Some(path) = release_package_site.as_ref().filter(|path| path.exists()) {
        return path.to_string_lossy().into_owned();
    }

    let cargo_leptos_site = std::path::Path::new("target/site");
    if cargo_leptos_site.exists() {
        return cargo_leptos_site.to_string_lossy().into_owned();
    }

    release_package_site
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|| String::from("target/site"))
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("dnsmasqweb=info,tower_http=warn"));

    tracing_subscriber::fmt().with_env_filter(filter).init();
}
