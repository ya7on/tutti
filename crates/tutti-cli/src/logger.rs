use std::hash::{DefaultHasher, Hash, Hasher};

use colored::{Color, Colorize};

pub struct Logger;

impl Logger {
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

    pub fn log(service_name: &str, message: &str) {
        let prefix = format!("[{service_name}]").color(Self::string_to_color(service_name));
        print!("{prefix} {message}");
    }
}
