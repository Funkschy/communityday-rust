use std::{
    collections::HashSet,
    error,
    io::{self, Write},
    sync::mpsc,
    time::Duration,
};

use curl::easy::Easy;
use num_cpus;
use threadpool::ThreadPool;
use url::Url;

mod cli;
mod html;
use html::LinkFinder;

pub const MAX_REDIRECT: u32 = 2;
pub const TIMEOUT_SEC: u64 = 2;

fn main() -> Result<(), Box<error::Error>> {
    // da viele Threads blockieren, per default die doppelte Anzahl der
    // verfuegbaren Kerne nutzen
    let n_workers = num_cpus::get() * 4;
    let (visited_in, visited_out) = mpsc::channel();
    let pool = ThreadPool::new(n_workers);

    // die start url in den channel senden
    visited_in.send(cli::get_url_from_command_line()?)?;
    let mut visited = HashSet::new();

    let out = io::stdout();
    // stdout nur einmal locken fuer mehr performance beim printen
    let mut handle = out.lock();

    for url in visited_out {
        // urls nicht doppelt besuchen
        if visited.contains(&url) {
            continue;
        }

        visited.insert(url.clone());
        handle.write_all(format!("Visited {}\n", &url).as_bytes())?;

        let visited_in = visited_in.clone();
        pool.execute(move || {
            for url in visit(url) {
                visited_in.send(url.clone()).unwrap();
            }
        });
    }

    Ok(())
}

/// Besucht eine Url, laedt den HTML code herunter und extrahiert alle Links aus diesem
fn visit(url: Url) -> Vec<Url> {
    let html = download_html(&url);
    if let Err(err) = html {
        eprintln!("Error: {}", err);
        return Vec::new();
    } else if !["http", "https", "file"].contains(&url.scheme()) {
        eprintln!("{} ist not supported", url.scheme());
        return Vec::new();
    }

    let lf = LinkFinder::get_links(url.into_string(), &html.unwrap());
    lf.collect_links()
}

fn download_html(url: &Url) -> Result<String, Box<error::Error>> {
    let mut dst = Vec::new();
    let mut easy = Easy::new();
    easy.url(url.as_str())?;
    easy.follow_location(MAX_REDIRECT > 0)?;
    easy.timeout(Duration::new(TIMEOUT_SEC, 0))?;
    easy.max_redirections(MAX_REDIRECT)?;

    let mut transfer = easy.transfer();
    transfer.write_function(|data| {
        dst.extend_from_slice(data);
        Ok(data.len())
    })?;
    transfer.perform()?;
    std::mem::drop(transfer);

    Ok(String::from_utf8_lossy(&dst).to_string())
}
