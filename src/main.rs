mod config;
mod db;
mod file;
mod http;
mod util;

use config::Config;

use futures::future;
use signal_hook::consts::TERM_SIGNALS;
use signal_hook::iterator::Signals;
use tokio_stream::wrappers::UnixListenerStream;

#[tokio::main]
async fn main() {
    let config = Config::init();

    config.print();

    let db = match db::open(config.db_path.clone()).await {
        Ok(db) => db,
        Err(_) => {
            eprintln!("Error: failed to open database {}", config.db_path);
            std::process::exit(1);
        }
    };

    let listener = match file::setup_dirs_get_listener(&config.socket_path).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let incoming = UnixListenerStream::new(listener);

    let server = http::create_server(db, &config, incoming);

    let sigwait = tokio::spawn(async move {
        term_signal().await
    });

    future::select(server, sigwait).await;

    util::cleanup(&config);
}

async fn term_signal() {
    let mut signals = Signals::new(TERM_SIGNALS).unwrap();

    for _ in &mut signals {
        eprintln!("Received signal, terminating.");
        break;
    }
}
