use clap::{Arg, App};
use std::sync::Arc;

#[derive(Clone)]
pub struct Config {
    pub socket_path: Arc<str>,
    pub db_path:     Arc<str>,
    protocol:        Arc<str>,
    hostname:        Arc<str>,
    port:            Arc<str>,
    download_path:   Arc<str>,
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ socket_path: '{}', protocol: '{}', hostname: '{}', port: '{}' }}",
            self.socket_path, self.protocol, self.hostname, self.port)
    }
}

impl Config {
    pub fn init() -> Self {
        // TODO add description to each option
        let matches = App::new("urlnao")
        .arg(Arg::with_name("socket_path")
            .short("s")
            .long("socket-path")
            .takes_value(true)
            .default_value("./urlnao.sock"))
        .arg(Arg::with_name("db_path")
            .long("db-path")
            .takes_value(true)
            .default_value("./db"))
        .arg(Arg::with_name("hostname")
            .short("h")
            .long("hostname")
            .takes_value(true)
            .default_value("localhost"))
        .arg(Arg::with_name("port")
            .long("port")
            .takes_value(true)
            .default_value(""))
        .arg(Arg::with_name("protocol")
            .long("protocol")
            .takes_value(true)
            .possible_values(&["http", "https"])
            .default_value("http"))
        .arg(Arg::with_name("download_path")
            .long("download-path")
            .takes_value(true)
            .default_value("f"))
        .get_matches();

        config_to_struct(matches)
    }

    pub fn get_url_prefix(&self) -> String {
        format!("{}://{}{}/{}", self.protocol, self.hostname, self.port, self.download_path)
    }
}

fn config_to_struct(matches: clap::ArgMatches<'_>) -> Config {
    Config {
        socket_path:   Arc::from(matches.value_of("socket_path").unwrap_or("./urlnao.sock")),
        db_path:       Arc::from(matches.value_of("db_path").unwrap_or("./db")),
        hostname:      Arc::from(matches.value_of("hostname").unwrap_or("localhost")),
        port:          Arc::from(matches.value_of("port").unwrap_or("23523")),
        protocol:      Arc::from(matches.value_of("protocol").unwrap_or("http")),
        download_path: Arc::from(matches.value_of("download_path").unwrap_or("f")),
    }
}
