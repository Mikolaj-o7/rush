use reedline::{ Prompt, PromptEditMode, PromptHistorySearch, FileBackedHistory, Reedline, Signal };
use std::borrow::Cow;
use std::env;
use std::io::Write;
use std::process::{ Command, Stdio };

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

    fn render_prompt_history_search_indicator(
        &self,
        _history_search: PromptHistorySearch
    ) -> Cow<str> {
        Cow::Borrowed("⎈ ")
    }
}

#[derive(Debug, PartialEq, Clone)]
enum Token {
    Command(String),
    Argument(String),
    StringLiteral(String),
    Whitespace,
    Newline,
    Illegal(String),
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    is_first_token: bool,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
            is_first_token: true,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.position >= self.input.len() {
            return Token::Newline;
        }

        let ch = self.input[self.position];
        self.position += 1;

        match ch {
            '"' | '\'' => self.tokenize_string(ch),
            ch if ch.is_whitespace() => Token::Whitespace,
            _ => self.tokenize_word(ch),
        }
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.input[self.position].is_whitespace() {
            self.position += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn tokenize_string(&mut self, quote: char) -> Token {
        let mut content = String::new();
        while let Some(ch) = self.peek() {
            if ch == quote {
                self.position += 1;
                return Token::StringLiteral(content);
            }
            content.push(ch);
            self.position += 1;
        }
        Token::Illegal(format!("Unclosed quote: {}", quote))
    }

    fn tokenize_word(&mut self, first_char: char) -> Token {
        let mut word = String::from(first_char);
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() || ch == '"' || ch == '\'' {
                break;
            }
            word.push(ch);
            self.position += 1;
        }
        if self.is_first_token {
            self.is_first_token = false;
            Token::Command(word)
        } else {
            Token::Argument(word)
        }
    }
}

fn main() {
    let history = Box::new(
        FileBackedHistory::with_file(1000, "rush_history.txt".into()).expect(
            "Failed to create history file"
        )
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

                let mut lexer = Lexer::new(input.to_string());
                let mut tokens = Vec::new();
                loop {
                    let token = lexer.next_token();
                    if token == Token::Newline {
                        break;
                    }
                    tokens.push(token);
                }

                let mut cmd = None;
                let mut args = Vec::new();
                for token in tokens {
                    match token {
                        Token::Illegal(msg) => {
                            eprintln!("rush: {}", msg);
                            continue;
                        }
                        Token::Command(s) => cmd = Some(s),
                        Token::Argument(s) | Token::StringLiteral(s) => args.push(s),
                        Token::Whitespace => {}
                        Token::Newline => unreachable!(),
                    }
                }

                let cmd = match cmd {
                    Some(c) => c,
                    None => {
                        continue;
                    }
                };

                match cmd.as_str() {
                    "exit" => {
                        println!("Exiting rush...");
                        break;
                    }
                    "cd" => {
                        let new_dir = args.get(0).map_or_else(
                            || env::var("HOME").unwrap_or_else(|_| "/".into()),
                            |s| s.to_string()
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

                let result = Command::new(&cmd)
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
