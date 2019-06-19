use std::{collections::HashSet, error};

use clap::{App, Arg};
use curl::easy::Easy;
use url::Url;

mod html;
mod thread;
use html::LinkFinder;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
pub const ABOUT: &str = env!("CARGO_PKG_DESCRIPTION");

fn main() -> Result<(), Box<error::Error>> {
    let url = get_url_from_command_line()?;

    let mut child_urls = visit(url)?;
    let mut visited = HashSet::new();

    while !child_urls.is_empty() {
        let child = child_urls.pop().unwrap();
        if visited.contains(&child) {
            continue;
        }
        println!("{}", &child);

        visited.insert(child.clone());
        child_urls.append(&mut visit(child)?);
    }

    Ok(())
}

fn get_url_from_command_line() -> Result<Url, url::ParseError> {
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

fn visit(url: Url) -> Result<Vec<Url>, Box<error::Error>> {
    let html = get_html(&url);
    if html.is_err() {
        println!("{} Has malformed html", url);
        return Ok(Vec::new());
    } else if !["http", "https", "file"].contains(&url.scheme()) {
        println!("Could not download {}", url);
        return Ok(Vec::new());
    }

    let lf = LinkFinder::get_links(url.into_string(), &html?);
    Ok(lf
        .link_strings
        .iter()
        .filter_map(|url| lf.get_url(*url))
        .collect())
}

fn get_html(url: &Url) -> Result<String, Box<error::Error>> {
    let mut dst = Vec::new();
    let mut easy = Easy::new();
    easy.url(url.as_str())?;
    easy.follow_location(true)?;

    let mut transfer = easy.transfer();
    transfer.write_function(|data| {
        dst.extend_from_slice(data);
        Ok(data.len())
    })?;
    transfer.perform()?;
    std::mem::drop(transfer);

    Ok(String::from_utf8_lossy(&dst).to_string())
}
