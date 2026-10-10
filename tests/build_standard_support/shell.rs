//! Reads the shell lines `make -n` prints: which commands a logical line runs, and which of them
//! compile or run code, so the contract can ask each one to assign `RUSTFLAGS`.

/// Cargo subcommands that inspect or install and compile nothing, so a recipe running
/// one needs no `RUSTFLAGS`. Every other Cargo subcommand, and Whitaker, compiles or runs
/// code, and a development or held-out recipe must assign the flags it takes there.
const INSPECTION_SUBCOMMANDS: &[&str] = &[
    "fmt", "metadata", "doc", "install", "binstall", "audit", "deny", "machete", "version", "help",
];

/// Returns whether a command runs a tool that compiles or runs code: Whitaker, or Cargo with a
/// subcommand outside the inspection list. A version probe (`--version`, `-V`) runs nothing.
///
/// ```text
/// compiles("cargo +nightly clippy --all-targets") -> true
/// compiles("whitaker --all")                      -> true
/// compiles("/home/u/.cargo/bin/cargo test")       -> true (Cargo exports its own path)
/// compiles("cargo nextest --version")             -> false
/// compiles("cargo fmt --all --check")             -> false
/// ```
pub fn compiles(command: &str) -> bool {
    let words: Vec<&str> = command.split_whitespace().collect();
    if words.iter().any(|word| matches!(*word, "--version" | "-V")) {
        return false;
    }
    let cargo_subcommand = words
        .iter()
        .position(|word| names_program(word, "cargo"))
        .and_then(|at| words.get(at + 1..))
        .and_then(|rest| {
            rest.iter()
                .find(|word| !word.starts_with('+') && !word.starts_with('-'))
        })
        .is_some_and(|sub| !INSPECTION_SUBCOMMANDS.contains(sub));
    cargo_subcommand || words.iter().any(|word| names_program(word, "whitaker"))
}

/// Returns whether a word names a program: its final path component, with or without `.exe`,
/// matches (`cargo`, `/usr/bin/cargo`, `C:\\tools\\cargo.exe`).
pub fn names_program(word: &str, program: &str) -> bool {
    word.rsplit(['/', '\\'])
        .next()
        .is_some_and(|name| name == program || name.strip_suffix(".exe") == Some(program))
}

/// Splits one logical line of `make -n` output into the shell commands it runs: at `;`, `&&`,
/// `||` and a newline, outside quotes, so a probe, a `cargo metadata` and the real command that
/// share a line are judged one by one.
///
/// ```text
/// shell_commands("a && b; c \"x;y\"") -> ["a ", " b", " c \"x;y\""]
/// ```
pub fn shell_commands(line: &str) -> Vec<String> { ShellCommandParser::new(line).parse() }

#[derive(Clone, Copy)]
struct ShellContext {
    quote: Option<char>,
    parenthesis_depth: usize,
    is_substitution: bool,
}

impl ShellContext {
    const fn top_level() -> Self {
        Self {
            quote: None,
            parenthesis_depth: 0,
            is_substitution: false,
        }
    }

    const fn substitution() -> Self {
        Self {
            quote: None,
            parenthesis_depth: 1,
            is_substitution: true,
        }
    }
}

struct ShellCommandParser {
    characters: Vec<char>,
    commands: Vec<String>,
    current: String,
    contexts: Vec<ShellContext>,
    index: usize,
}

impl ShellCommandParser {
    fn new(line: &str) -> Self {
        Self {
            characters: line.chars().collect(),
            commands: Vec::new(),
            current: String::new(),
            contexts: vec![ShellContext::top_level()],
            index: 0,
        }
    }

    fn parse(mut self) -> Vec<String> {
        while let Some(character) = self.characters.get(self.index).copied() {
            self.consume(character);
        }
        self.commands.push(self.current);
        self.commands
    }

    fn consume(&mut self, character: char) {
        if self.consume_escape(character) || self.consume_quoted(character) {
            return;
        }
        if self.consume_substitution(character) || self.consume_delimiter(character) {
            return;
        }
        self.current.push(character);
        self.index += 1;
    }

    fn consume_escape(&mut self, character: char) -> bool {
        if character != '\\' || self.active_quote() == Some('\'') {
            return false;
        }
        self.current.push(character);
        self.index += 1;
        if let Some(escaped) = self.characters.get(self.index).copied() {
            self.current.push(escaped);
            self.index += 1;
        }
        true
    }

    fn consume_quoted(&mut self, character: char) -> bool {
        let Some(active_quote) = self.active_quote() else {
            return false;
        };
        if active_quote == '"' && self.is_substitution_start(character) {
            self.open_substitution();
            return true;
        }
        self.current.push(character);
        if character == active_quote {
            self.update_context(|context| context.quote = None);
        }
        self.index += 1;
        true
    }

    fn consume_substitution(&mut self, character: char) -> bool {
        if !self.is_substitution_start(character) {
            return false;
        }
        self.open_substitution();
        true
    }

    fn consume_delimiter(&mut self, character: char) -> bool {
        let Some(context) = self.contexts.last().copied() else {
            return false;
        };
        match character {
            '\'' | '"' => {
                self.update_context(|active_context| active_context.quote = Some(character));
                self.current.push(character);
                self.index += 1;
                true
            }
            '(' => {
                self.update_context(|active_context| active_context.parenthesis_depth += 1);
                self.current.push(character);
                self.index += 1;
                true
            }
            ')' if context.parenthesis_depth > 0 => {
                self.update_context(|active| active.parenthesis_depth -= 1);
                self.current.push(character);
                if context.is_substitution && context.parenthesis_depth == 1 {
                    self.close_substitution();
                }
                self.index += 1;
                true
            }
            ';' if self.is_top_level(context) => {
                self.commands.push(std::mem::take(&mut self.current));
                self.index += 1;
                true
            }
            '&' | '|' if self.is_top_level(context) => {
                self.consume_boolean_operator(character);
                true
            }
            _ => false,
        }
    }

    fn consume_boolean_operator(&mut self, character: char) {
        if self.next_character() == Some(character) {
            self.commands.push(std::mem::take(&mut self.current));
            self.index += 2;
        } else {
            self.current.push(character);
            self.index += 1;
        }
    }

    fn open_substitution(&mut self) {
        self.current.push('$');
        self.current.push('(');
        self.contexts.push(ShellContext::substitution());
        self.index += 2;
    }

    fn close_substitution(&mut self) {
        let remaining = self.contexts.len().saturating_sub(1);
        self.contexts.truncate(remaining);
    }

    const fn is_top_level(&self, context: ShellContext) -> bool {
        self.contexts.len() == 1 && context.parenthesis_depth == 0
    }

    fn active_quote(&self) -> Option<char> {
        self.contexts.last().and_then(|context| context.quote)
    }

    fn update_context(&mut self, update: impl FnOnce(&mut ShellContext)) {
        if let Some(context) = self.contexts.last_mut() {
            update(context);
        }
    }

    fn next_character(&self) -> Option<char> {
        self.index
            .checked_add(1)
            .and_then(|index| self.characters.get(index))
            .copied()
    }

    fn is_substitution_start(&self, character: char) -> bool {
        character == '$' && self.next_character() == Some('(')
    }
}

/// Returns a command without the shell keywords that lead it: `then`, `else`, `do` and the like.
pub fn without_leading_keywords(command: &str) -> &str {
    let mut rest = command.trim();
    while let Some((word, after)) = rest.split_once(char::is_whitespace) {
        if matches!(
            word,
            "if" | "then" | "else" | "elif" | "do" | "while" | "until" | "!" | "{" | "("
        ) {
            rest = after.trim_start();
        } else {
            break;
        }
    }
    rest
}

/// Returns the leading `NAME=value` assignments of a command line, where a value is double quoted,
/// single quoted or a bare word, up to the first word that is not an assignment.
///
/// ```text
/// leading_assignments("A=\"x y\" B=z cargo test") == [("A", "x y"), ("B", "z")]
/// leading_assignments("cargo test")               == []
/// ```
pub fn leading_assignments(command: &str) -> Vec<(String, String)> {
    let mut rest = command.trim_start();
    let mut found = Vec::new();
    while let Some((name, after)) = rest.split_once('=') {
        let is_name = name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        if !is_name {
            break;
        }
        let (value, remainder) = split_value(after);
        found.push((name.to_owned(), value.to_owned()));
        rest = remainder.trim_start();
    }
    found
}

/// Splits the text after an `=` into a quoted or bare value and the rest.
fn split_value(after: &str) -> (&str, &str) {
    let quoted = |quote: char| {
        after
            .strip_prefix(quote)
            .map(|body| body.split_once(quote).unwrap_or((body, "")))
    };
    quoted('"')
        .or_else(|| quoted('\''))
        .unwrap_or_else(|| after.split_once(char::is_whitespace).unwrap_or((after, "")))
}

#[cfg(test)]
mod shell_command_tests {
    //! Keep shell separators inside quoted and nested command substitutions.

    use super::shell_commands;

    #[test]
    fn nested_substitution_separators_stay_inside_the_assignment() {
        let line = concat!(
            "CARGO_ENCODED_RUSTFLAGS=\"$(filtered=$(bash -c ",
            "'set -euo pipefail; printf \\\"%s\\\" value')); ",
            "if [ -n \"$filtered\" ]; then printf '%s' \"$filtered\"; fi)\" ",
            "RUSTFLAGS=\"-D warnings -Zthreads=8\" cargo build"
        );

        assert_eq!(shell_commands(line), vec![line.to_owned()]);
    }
}
