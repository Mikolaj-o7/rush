use std::env;
use std::io::{self, Write};
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

fn main() -> io::Result<()> {
    loop {
        let username = get_username();
        let current_directory_path = get_display_path();
        print!("{username}@rush:{current_directory_path} ❯ ");
                
        io::stdout().flush().unwrap();

        let mut input = String::new();
        let bytes_read = io::stdin().read_line(&mut input)?;

        if bytes_read == 0 {
            println!();
            break;
        }

        let input = input.trim();

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
                let new_dir = args.get(0).map_or_else(|| env::var("HOME").unwrap_or_else(|_| "/".into()), |s| s.to_string());
                
                if let Err(e) = env::set_current_dir(&new_dir) {
                    eprintln!("rush: cd: {}: {}", new_dir, e);
                }
                
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

            Err(e) => {
                eprintln!("rush: command not found: {}", cmd);
            }
        }
    }

    Ok(())
}
