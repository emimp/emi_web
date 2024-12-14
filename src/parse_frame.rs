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

impl Buffer {
    fn parse_area(data: &str) -> Rect {
        let re = Regex::new(r"x: (\d+), y: (\d+), width: (\d+), height: (\d+)").unwrap();
        if let Some(caps) = re.captures(data) {
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
        let re = Regex::new(r"x: (\d+), y: (\d+), fg: (\w+)").unwrap();
        let mut styles = Vec::new();

        for caps in re.captures_iter(data) {
            let style = Style {
                x: caps[1].parse().unwrap(),
                y: caps[2].parse().unwrap(),
                fg: caps[3].to_string(),
            };
            styles.push(style);
        }

        styles
    }

    pub fn from_string(buffer_str: &str) -> Self {
        // Corrected regex to handle multiline data properly
        let area_re = Regex::new(r"area: Rect \{([\s\S]+?)\}").expect("Invalid regex for area");
        let styles_re = Regex::new(r"styles: \[([\s\S]+?)\]").expect("Invalid regex for styles");
        let content_re = Regex::new(r"content: \[([\s\S]+?)]").expect("Invalid regex for content");

        // Debugging output

        let area_data = match area_re.captures(buffer_str) {
            Some(caps) => caps
                .get(1)
                .map_or_else(|| "".to_string(), |m| m.as_str().to_string()),
            None => {
                eprintln!("Failed to parse area data, raw input: {}", buffer_str);
                println!("Buffer Input:\n{}", buffer_str);
                String::new()
            }
        };

        let styles_data = match styles_re.captures(buffer_str) {
            Some(caps) => caps
                .get(1)
                .map_or_else(|| "".to_string(), |m| m.as_str().to_string()),
            None => {
                eprintln!("Failed to parse styles data, raw input: {}", buffer_str);
                String::new()
            }
        };

        let content: Vec<String> = match content_re.captures(buffer_str) {
            Some(caps) => caps
                .get(1)
                .map_or_else(|| "".to_string(), |m| m.as_str().to_string())
                .split(",\n")
                .map(|s| s.trim().to_string())
                .collect(),
            None => {
                eprintln!("Failed to parse content data, raw input: {}", buffer_str);
                Vec::new()
            }
        };

        // Debugging parsed data

        let area = Buffer::parse_area(&area_data);
        let styles = Buffer::parse_styles(&styles_data);

        Buffer {
            area,
            content,
            styles,
        }
    }
}
