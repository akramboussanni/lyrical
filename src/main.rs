mod client;
mod ui;

use std::collections::HashMap;
use std::io;
use clap::Parser;
use std::io::{Write, stdin, stdout};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

const PAGE_SIZE: usize = 5;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    title: Option<String>,
    #[arg(short, long)]
    query: Option<String>,
    #[arg(long)]
    artist: Option<String>,
    #[arg(long)]
    album: Option<String>,
}

fn main() -> io::Result<()>{
    let args = Args::parse();
    let mut opt: HashMap<String, String>;
    let auto = args.title.is_some() || args.query.is_some();

    if !auto {
        opt = HashMap::new();
        opt.insert("q".to_string(), prompt());
    } else {
        opt = args_to_map(&args)
    }

    match client::request(opt) {
        Ok(responses) => {
            enable_raw_mode()?;
            let resp = ui::paginate(&responses);
            disable_raw_mode()?;

            match resp {
                Ok(lyrics) => {
                    ui::show_lyrics(lyrics).unwrap();
                }
                Err(e) => eprintln!("Pagination error: {}", e),
            }

            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            Ok(())
        }
    }

}

fn prompt() -> String {
    println!("Enter keywords to look for a song (title, artist, album):");
    stdout().flush().unwrap();

    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();

    input
}

fn args_to_map(args: &Args) -> HashMap<String, String> {
    let mut map = HashMap::new();

    if let Some(title) = &args.title {
        map.insert("title".to_string(), title.clone());
    }
    if let Some(query) = &args.query {
        map.insert("query".to_string(), query.clone());
    }
    if let Some(artist) = &args.artist {
        map.insert("artist".to_string(), artist.clone());
    }
    if let Some(album) = &args.album {
        map.insert("album".to_string(), album.clone());
    }

    map
}
