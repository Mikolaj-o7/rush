use std::env::{self};
use std::io::{self, Write};

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

        println!("You typed: {}", input);
    }

    Ok(())
}
