use std::{io::{self, Write, stdout}, time::Duration};
use crossterm::{
    cursor, event::{poll, read, Event, KeyCode},
    execute, queue,
    style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode},
};
use crossterm::event::KeyEventKind;
use regex::Regex;
use crate::{client::Response, PAGE_SIZE};

pub fn paginate(responses: &[Response]) -> io::Result<String> {
    let filtered: Vec<&Response> = responses
        .iter()
        .filter(|x| !x.instrumental && x.synced_lyrics.is_some())
        .collect();
    if filtered.is_empty() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "No matching songs"));
    }

    let mut page_num = 0;
    let mut selected = 0;
    let max_pages = (filtered.len() + PAGE_SIZE - 1) / PAGE_SIZE;

    let mut stdout = stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    // Draw first time
    let mut need_redraw = true;

    loop {
        if need_redraw {
            queue!(
                stdout,
                cursor::MoveTo(0, 0),
                Clear(ClearType::All)
            )?;

            let start = page_num * PAGE_SIZE;
            let end = (start + PAGE_SIZE).min(filtered.len());

            for (i, song) in filtered[start..end].iter().enumerate() {
                if i == selected {
                    queue!(
                        stdout,
                        SetBackgroundColor(Color::Blue),
                        SetForegroundColor(Color::White)
                    )?;
                }
                writeln!(
                    stdout,
                    "{}. {} - {} ({}) [{:.1}s]",
                    i + 1,
                    song.artist_name,
                    song.track_name,
                    song.album_name,
                    song.duration
                )?;
                if i == selected {
                    queue!(stdout, ResetColor)?;
                }
            }

            writeln!(stdout)?;
            writeln!(stdout, "Page {}/{}", page_num + 1, max_pages)?;
            writeln!(stdout, "[↑/↓] Move  [←/→] Page  [Enter] Select  [Esc] Quit")?;

            stdout.flush()?;
            need_redraw = false;
        }

        if poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = read()? {
                if key_event.kind == KeyEventKind::Press {
                    let mut changed = false;
                    match key_event.code {
                        KeyCode::Up if selected > 0 => {
                            selected -= 1;
                            changed = true;
                        }
                        KeyCode::Down => {
                            let start = page_num * PAGE_SIZE;
                            let end = (start + PAGE_SIZE).min(filtered.len());
                            if selected + 1 < end - start {
                                selected += 1;
                                changed = true;
                            }
                        }
                        KeyCode::Left if page_num > 0 => {
                            page_num -= 1;
                            selected = 0;
                            changed = true;
                        }
                        KeyCode::Right if page_num + 1 < max_pages => {
                            page_num += 1;
                            selected = 0;
                            changed = true;
                        }
                        KeyCode::Enter => {
                            disable_raw_mode()?;
                            execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
                            let idx = page_num * PAGE_SIZE + selected;
                            return Ok(filtered[idx].synced_lyrics.clone().unwrap());
                        }
                        KeyCode::Esc => {
                            disable_raw_mode()?;
                            execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
                            return Err(io::Error::new(io::ErrorKind::Interrupted, "User aborted"));
                        }
                        _ => {}
                    }
                    if changed {
                        need_redraw = true;
                    }
                }
            }
        }
    }
}

pub fn show_lyrics(lyrics: String) -> io::Result<()> {
    let re = Regex::new(r"\[(\d{2}:\d{2}\.\d{2})]\s*(.+)").unwrap();
    let lines: Vec<&str> = lyrics.lines().collect();

    let mut stdout = stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, cursor::Hide, Clear(ClearType::All), cursor::MoveTo(0, 0))?;

    for i in 0..lines.len().saturating_sub(1) {
        if let (Some(caps), Some(next_caps)) = (re.captures(lines[i]), re.captures(lines[i + 1])) {
            let t0 = parse_timestamp(caps.get(1).unwrap().as_str());
            let t1 = parse_timestamp(next_caps.get(1).unwrap().as_str());
            let text = caps.get(2).unwrap().as_str();
            let delay = ((t1 - t0) / text.chars().count() as f64 * 1000.0) as u64;

            for ch in text.chars() {
                print!("{}", ch);
                stdout.flush()?;
                std::thread::sleep(Duration::from_millis(delay));
            }
            println!();
        }
    }

    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
    Ok(())
}

fn parse_timestamp(ts: &str) -> f64 {
    let parts: Vec<&str> = ts.split(':').collect();
    let minutes: u32 = parts[0].parse().unwrap_or(0);
    let sec_parts: Vec<&str> = parts[1].split('.').collect();
    let seconds: u32 = sec_parts[0].parse().unwrap_or(0);
    let hundredths: u32 = sec_parts[1].parse().unwrap_or(0);
    (minutes * 60) as f64 + seconds as f64 + hundredths as f64 / 100.0
}