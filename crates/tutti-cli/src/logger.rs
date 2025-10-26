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
        let _ = writeln!(self.output, "{prefix} {message}");
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

        logger.log("test", "message");

        let output = String::from_utf8(logger.output.into_inner()).unwrap();
        assert!(output.contains("[test]"));
        assert!(output.contains("message"));
    }
}
