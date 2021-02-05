use clap::{Arg, App};
use std::sync::Arc;

pub enum SuffixType {
    ShortID,
    FileName,
}

#[derive(Clone)]
pub struct Config {
    pub socket_path: Arc<str>,
    pub db_path:     Arc<str>,
    protocol:        Arc<str>,
    hostname:        Arc<str>,
    port:            Arc<str>,
    shortid_path:    Arc<str>,
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
        let matches = App::new("urlnao")
            .version("0.1.0")
            .about("Upload service for file sharing with weechat-android")
            .arg(Arg::with_name("socket_path")
                .short("s")
                .long("socket-path")
                .takes_value(true)
                .help("Path to the Unix domain socket")
                .default_value("./urlnao.sock"))
            .arg(Arg::with_name("db_path")
                .long("db-path")
                .takes_value(true)
                .help("Path to urlnao's key-value store")
                .default_value("./db"))
            .arg(Arg::with_name("hostname")
                .short("h")
                .long("hostname")
                .takes_value(true)
                .help("Public hostname under which\nurlnao is reachable")
                .default_value("localhost"))
            .arg(Arg::with_name("port")
                .long("port")
                .takes_value(true)
                .help("Optional non-standard port under\nwhich urlnao is reachable")
                .default_value(""))
            .arg(Arg::with_name("protocol")
                .long("protocol")
                .takes_value(true)
                .possible_values(&["http", "https"])
                .help("Protocol under which urlnao\nis reachable")
                .default_value("http"))
            .arg(Arg::with_name("shortid_path")
                .long("shortid-path")
                .takes_value(true)
                .help("URL path under which files should\nbe reachable by their short id")
                .default_value("f"))
            .arg(Arg::with_name("download_path")
                .long("download-path")
                .takes_value(true)
                .help("URL path under which files should\nbe reachable by their original name")
                .default_value("d"))
            .get_matches();

        config_to_struct(matches)
    }

    pub fn prepend_url(&self, stype: SuffixType, suffix: &str) -> String {
        match (self.port.len(), stype) {
            (0, SuffixType::FileName) => format!("{}://{}/{}/{}",
                self.protocol, self.hostname, self.download_path, suffix),
            (0, _) => format!("{}://{}/{}/{}",
                self.protocol, self.hostname, self.shortid_path, suffix),
            (_, SuffixType::FileName) => format!("{}://{}:{}/{}/{}",
                self.protocol, self.hostname, self.port, self.download_path, suffix),
            (_, _) => format!("{}://{}:{}/{}/{}",
                self.protocol, self.hostname, self.port, self.shortid_path, suffix),
        }
    }
    }
}

fn config_to_struct(matches: clap::ArgMatches<'_>) -> Config {
    Config {
        socket_path:   Arc::from(matches.value_of("socket_path").unwrap_or("./urlnao.sock")),
        db_path:       Arc::from(matches.value_of("db_path").unwrap_or("./db")),
        hostname:      Arc::from(matches.value_of("hostname").unwrap_or("localhost")),
        port:          Arc::from(matches.value_of("port").unwrap_or("23523")),
        protocol:      Arc::from(matches.value_of("protocol").unwrap_or("http")),
        shortid_path:  Arc::from(matches.value_of("shortid_path").unwrap_or("f")),
        download_path: Arc::from(matches.value_of("download_path").unwrap_or("d")),
    }
}
