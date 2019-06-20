use clap::{App, Arg};
use url::Url;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
pub const ABOUT: &str = env!("CARGO_PKG_DESCRIPTION");

/// Liest die Url, die beim start des Programmes mit -u oder --url uebergeben wurde
pub fn get_url_from_command_line() -> Result<Url, url::ParseError> {
    let matches = App::new(NAME)
        .version(VERSION)
        .author(AUTHOR)
        .about(ABOUT)
        .arg(
            Arg::with_name("url")
                .short("u")
                .long("url")
                .value_name("URL")
                .help("The starting URL")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    Url::parse(matches.value_of("url").unwrap())
}
