mod client;
mod ui;

use std::collections::HashMap;
use std::io;
use clap::Parser;
use std::io::{Write, stdin, stdout, Read};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crate::client::Response;

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

    #[arg(short, long)]
    debug: bool
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let debug = args.debug;
    let auto = args.title.is_some() || args.query.is_some();

    if debug && !auto {
        println!("--- DEBUG MODE ENABLED ---");
    }

    let opt = if auto {
        args_to_map(&args)
    } else {
        let mut map = HashMap::new();
        map.insert("q".to_string(), prompt());
        map
    };

    println!("\nSearching...");
    match client::request(opt) {
        Ok(responses) => handle_responses(responses, auto, debug),
        Err(e) => {
            eprintln!("Error: {}", e);
            Ok(())
        }
    }
}

fn handle_responses(responses: Vec<Response>, auto: bool, debug: bool) -> io::Result<()> {
    if auto {
        match responses.first().and_then(|r| r.synced_lyrics.as_deref()) {
            Some(lyrics) => ui::show_lyrics(lyrics.to_string(), debug)?,
            None => {
                eprintln!(
                    "{}",
                    if responses.is_empty() {
                        "No result was found for the provided search."
                    } else {
                        "No synced lyrics found."
                    }
                );
                pause();
            }
        }

        return Ok(())
    }

    enable_raw_mode()?;
    let result = ui::paginate(&responses);
    disable_raw_mode()?;

    match result {
        Ok(lyrics) => ui::show_lyrics(lyrics, debug)?,
        Err(e) => {
            eprintln!("{}", e);
            pause();
        }
    }
    
    Ok(())
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

fn pause() {
    println!("Press any key to continue...");
    let _ = stdin().read(&mut [0u8]).unwrap();
}