use colored::{Color, Colorize};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{self, Stdout, Write};

pub struct Logger<W: Write = Stdout> {
    output: W,
}

impl<W: Write> Logger<W> {
    pub fn new(output: W) -> Self {
        Self { output }
    }

    fn string_to_color(s: &str) -> Color {
        let colors = [
            Color::Green,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::BrightGreen,
            Color::BrightBlue,
            Color::BrightMagenta,
            Color::BrightCyan,
        ];

        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        let hash = hasher.finish();

        let idx = usize::try_from(hash).unwrap_or_default() % colors.len();
        colors[idx]
    }

    pub fn log(&mut self, service_name: &str, message: &str) {
        let prefix = format!("[{service_name}]").color(Self::string_to_color(service_name));
        for line in message.lines() {
            let _ = writeln!(self.output, "{prefix} {line}");
        }
    }

    pub fn system(&mut self, message: &str) {
        let prefix = "[system]".color(Color::Red);
        for line in message.lines() {
            let _ = writeln!(self.output, "{prefix} {line}");
        }
    }
}

impl Logger {
    pub fn default() -> Self {
        Self::new(io::stdout())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_log() {
        let buffer = Vec::new();
        let mut logger = Logger::new(Cursor::new(buffer));

        logger.log("test", "line1\nline2");

        let output = String::from_utf8(logger.output.into_inner()).unwrap();
        let service = "[test]".color(Color::BrightGreen);
        let line1 = format!("{service} line1");
        let line2 = format!("{service} line2");
        assert_eq!(output, format!("{line1}\n{line2}\n"));
    }

    #[test]
    fn test_log_default() {
        let _logger = Logger::default();
    }
}
