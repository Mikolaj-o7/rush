use reedline::{Prompt, PromptEditMode, PromptHistorySearch, FileBackedHistory, Reedline, Signal};
use std::borrow::Cow;
use std::env;
use std::io::Write;
use std::process::{Command, Stdio};

fn get_username() -> String {
    env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".into())
}

fn get_display_path() -> String {
    let cwd = env::current_dir().unwrap_or_default();
    let home = env::var("HOME").unwrap_or_default();
    let cwd_str = cwd.to_string_lossy();

    if cwd_str.starts_with(&home) {
        cwd_str.replacen(&home, "~", 1)
    } else {
        cwd_str.to_string()
    }
}

struct RushPrompt;

impl Prompt for RushPrompt {
    fn render_prompt_left(&self) -> Cow<str> {
        let username = get_username();
        let path = get_display_path();
        Cow::Owned(format!("{username}@rush:{path}"))
    }

    fn render_prompt_right(&self) -> Cow<str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, _prompt_mode: PromptEditMode) -> Cow<str> {
        Cow::Borrowed("$ ")
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<str> {
        Cow::Borrowed("⋙ ")
    }

    fn render_prompt_history_search_indicator(&self, _history_search: PromptHistorySearch) -> Cow<str> {
        Cow::Borrowed("⎈ ")
    }
}

fn main() {
    let history = Box::new(
        FileBackedHistory::with_file(1000, "rush_history.txt".into())
            .expect("Failed to create history file")
    );
    let mut editor = Reedline::create().with_history(history);
    let prompt = RushPrompt;
    
    loop {
                
        match editor.read_line(&prompt) {
            Ok(Signal::Success(line)) => {
                let input = line.trim();

                if input.is_empty() {
                    continue;
                }

                let mut parts = input.split_whitespace();
                let cmd = match parts.next() {
                    Some(c) => c,
                    None => continue,
                };
                let args: Vec<&str> = parts.collect();

                match cmd {
                    "exit" => {
                        println!("Exiting rush...");
                        break;
                    }
                    "cd" => {
                        let new_dir = args.get(0).map_or_else(
                            || env::var("HOME").unwrap_or_else(|_| "/".into()),
                            |s| s.to_string(),
                        );

                        if let Err(e) = env::set_current_dir(&new_dir) {
                            eprintln!("rush: cd: {}: {}", new_dir, e);
                        }

                        continue;
                    }
                    "clear" | "cls" => {
                        print!("\x1b[2J\x1b[3J\x1b[H");
                        std::io::stdout().flush().unwrap();
                        continue;
                    }
                    _ => {}
                }

                let result = Command::new(cmd)
                    .args(&args)
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .spawn();

                match result {
                    Ok(mut child) => {
                        let _ = child.wait();
                    }
                    Err(_) => {
                        eprintln!("rush: \"{}\" is not a recognized command", cmd);
                    }
                }
            }

            Ok(Signal::CtrlD) => {
                println!();
                break;
            }

            Ok(Signal::CtrlC) => {
                println!();
                continue;
            }

            Err(e) => {
                eprintln!("rush: error reading input: {}", e);
                break;
            }
        }
    }
}
