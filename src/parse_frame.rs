use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Style {
    x: i32,
    y: i32,
    fg: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Buffer {
    area: Rect,
    content: Vec<String>,
    styles: Vec<Style>,
}

// Precompiled regex patterns to avoid recompiling them multiple times
lazy_static::lazy_static! {
    static ref AREA_RE: Regex = Regex::new(r"x: (\d+), y: (\d+), width: (\d+), height: (\d+)").unwrap();
    static ref STYLES_RE: Regex = Regex::new(r"x: (\d+), y: (\d+), fg: (\w+)").unwrap();
    static ref BUFFER_RE: Regex = Regex::new(r"area: Rect \{([\s\S]+?)\}").unwrap();
    static ref STYLES_BUFFER_RE: Regex = Regex::new(r"styles: \[([\s\S]+?)\]").unwrap();
    static ref CONTENT_RE: Regex = Regex::new(r"content: \[([\s\S]+?)]").unwrap();
}

impl Buffer {
    fn parse_area(data: &str) -> Rect {
        // Matching area data and parsing fields
        if let Some(caps) = AREA_RE.captures(data) {
            Rect {
                x: caps[1].parse().unwrap(),
                y: caps[2].parse().unwrap(),
                width: caps[3].parse().unwrap(),
                height: caps[4].parse().unwrap(),
            }
        } else {
            Rect {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            }
        }
    }

    fn parse_styles(data: &str) -> Vec<Style> {
        // Capture and parse style data
        STYLES_RE
            .captures_iter(data)
            .map(|caps| Style {
                x: caps[1].parse().unwrap(),
                y: caps[2].parse().unwrap(),
                fg: caps[3].to_string(),
            })
            .collect()
    }

    pub fn from_string(buffer_str: &str) -> Self {
        // Use the precompiled regex to parse different parts
        let area_data = BUFFER_RE
            .captures(buffer_str)
            .and_then(|caps| caps.get(1).map(|m| m.as_str()))
            .unwrap_or("");

        let styles_data = STYLES_BUFFER_RE
            .captures(buffer_str)
            .and_then(|caps| caps.get(1).map(|m| m.as_str()))
            .unwrap_or("");

        let content: Vec<String> = CONTENT_RE
            .captures(buffer_str)
            .and_then(|caps| caps.get(1).map(|m| m.as_str()))
            .unwrap_or("")
            .split(",\n")
            .map(|s| {
                s.trim()
                    .trim_matches('"') // Remove surrounding quotes
                    .replace("\\\"", "\"") // Replace escaped quotes
                    .to_string()
            })
            .collect();

        let area = Buffer::parse_area(area_data);
        let styles = Buffer::parse_styles(styles_data);

        Buffer {
            area,
            content,
            styles,
        }
    }
}
