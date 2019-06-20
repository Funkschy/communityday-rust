use std::{collections::HashSet, error, sync::mpsc, time::Duration};

use curl::easy::Easy;
use threadpool::ThreadPool;
use url::Url;

mod cli;
mod html;
use html::LinkFinder;

pub const MAX_REDIRECT: u32 = 2;
pub const TIMEOUT_SEC: u64 = 2;

fn main() -> Result<(), Box<error::Error>> {
    let n_workers = 16;
    let (visited_in, visited_out) = mpsc::channel();
    let pool = ThreadPool::new(n_workers);

    visited_in.send(cli::get_url_from_command_line()?)?;
    let mut visited = HashSet::new();

    for url in visited_out {
        if visited.contains(&url) {
            continue;
        }

        visited.insert(url.clone());
        println!("Visited {}", &url);

        let visited_in = visited_in.clone();
        pool.execute(move || {
            if let Ok(urls) = visit(url) {
                for url in urls {
                    visited_in.send(url.clone()).unwrap();
                }
            }
        });
    }

    Ok(())
}

/// Besucht eine Url, laedt den HTML code herunter und extrahiert alle Links aus diesem
fn visit(url: Url) -> Result<Vec<Url>, Box<error::Error>> {
    let html = download_html(&url);
    if let Err(err) = html {
        println!("Error: {}", err);
        return Ok(Vec::new());
    } else if !["http", "https", "file"].contains(&url.scheme()) {
        println!("{} ist not supported", url.scheme());
        return Ok(Vec::new());
    }

    let lf = LinkFinder::get_links(url.into_string(), &html.unwrap());
    Ok(lf.collect_links())
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
