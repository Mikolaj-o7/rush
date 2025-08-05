use std::env;
use std::io::{self, Write};
use std::process::{Command, Stdio};

fn main() -> io::Result<()> {
    loop {
        print!("rush> ");
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

        if cmd == "exit" {
            println!("Exiting rush...");
            break;
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
