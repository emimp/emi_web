use std::{
    fs::{self, File},
    io::{self, Write},
    time::Duration,
};

use ratatui::{
    backend::TestBackend,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    Frame, Terminal,
};

pub fn init(width: u16, height: u16) -> io::Result<()> {
    let backend = TestBackend::new(width, height); // Simulates a terminal with 80x24 size
    let terminal = ratatui::Terminal::new(backend).unwrap();
    let app_result = App::default().run(terminal);
    ratatui::restore();
    app_result
}

#[derive(Debug, Default)]
pub struct App {
    counter: u8,
    exit: bool,
}

impl App {
    /// Runs the application's main loop until the user quits
    pub fn run<T: ratatui::backend::Backend>(
        &mut self,
        mut terminal: Terminal<T>,
    ) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_web_key_event();
            let seconds = Duration::from_millis(10);
            std::thread::sleep(seconds);
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    /// Updates the application's state based on user input
    fn handle_web_key_event(&mut self) {
        // Only read from the file if necessary
        if let Ok(contents) = fs::read_to_string("key_log.txt") {
            let key = contents.as_str();
            match key {
                "ArrowLeft" => self.decrement_counter(),
                "ArrowRight" => self.increment_counter(),
                _ => {}
            }

            // Clear the file only after handling the event
            let mut file = File::create("key_log.txt").expect("Failed to create or open the file");
            file.write_all("".as_bytes()).expect("Failed to write to file");
        }
    }

    fn increment_counter(&mut self) {
        if self.counter != 255 {
            self.counter += 1
        }
    }

    fn decrement_counter(&mut self) {
        if self.counter != 0 {
            self.counter -= 1
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Counter App Tutorial ".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);

        // Only write to the file if the counter has changed
        if self.counter != 0 {
            frame_to_file(buf).expect("couldn't write frame to file");
        }
    }
}

fn frame_to_file(buf: &mut Buffer) -> io::Result<()> {
    // Use a more efficient serialization or only write when necessary
    let t = format!("{:?}", buf);
    let mut file = File::create("output.txt")?;
    file.write_all(t.as_bytes())
}