use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
    time::Duration,
};

use ratatui::{
    backend::TestBackend,
    buffer::Buffer,
    layout::Rect,
    symbols::border,
    widgets::{Block, Paragraph, Widget},
    Frame, Terminal,
};

use tui_big_text::{BigText, PixelSize};

pub fn tui_init(width: u16, height: u16, uuid: String) -> io::Result<()> {
    let backend = TestBackend::new(width, height); // Simulates a terminal with 80x24 size
    let terminal = ratatui::Terminal::new(backend).unwrap();
    let app_result = App::init(uuid).run(terminal);
    ratatui::restore();
    app_result
}

#[derive(Debug, Default)]
pub struct App {
    counter: u8,
    exit: bool,
    uuid: String,
}

impl App {
    fn init(uuid: String) -> App {
        App {
            counter: 0,
            exit: false,
            uuid,
        }
    }
    /// Runs the application's main loop until the user quits
    fn run<T: ratatui::backend::Backend>(&mut self, mut terminal: Terminal<T>) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_web_key_event();
            self.check_exit();
            std::thread::sleep(Duration::from_millis(100));
        }

        Ok(())
    }

    fn check_exit(&mut self) {
        let uuid = &self.uuid;
        if Path::new(&format!("temp/{uuid}.remove")).exists() {
            self.exit = true;
        }
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    /// Updates the application's state based on user input
    fn handle_web_key_event(&mut self) {
        // Only read from the file if necessary
        let path = &format!("temp/{}.kl", self.uuid);
        if let Ok(contents) = fs::read_to_string(path) {
            let key = contents.as_str();
            File::create(path).expect("Failed to create or open the file");

            match key {
                "ArrowLeft" => self.decrement_counter(),
                "ArrowRight" => self.increment_counter(),
                _ => {}
            }

            // Clear the file only after handling the event
            let mut file = File::create("key_log.txt").expect("Failed to create or open the file");
            file.write_all("".as_bytes())
                .expect("Failed to write to file");
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
        use ratatui::prelude::*;

        let top_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(area);

        let bottom_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(80), Constraint::Percentage(20)])
            .split(top_layout[1]);

        let _big_text = BigText::builder()
            .pixel_size(PixelSize::HalfHeight)
            .style(Style::new().blue())
            .lines(vec!["Hello I'm Emi!".into(), "~~~~~~~~~~~~~~".into()])
            .build()
            .render(top_layout[0], buf);

        let title = Line::from(" Counter App Tutorial ".bold());
        let instructions = Line::from(vec![" Decrement <Left> Increment <Right> Quit <Q> ".into()]);
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
            .render(bottom_layout[0], buf);

        // Only write to the file if the counter has changed
        Paragraph::new("https://stackoverflow.com/questions/71345070/creating-a-2d-color-gradient-based-on-rgb-values-in-matplotlib").block(Block::bordered()).render(bottom_layout[1], buf);

        frame_to_file(buf, &self.uuid).expect("couldn't write frame to file");
    }
}

fn frame_to_file(buf: &mut Buffer, uuid: &str) -> io::Result<()> {
    // Use a more efficient serialization or only write when necessary
    let t = format!("{:?}", buf);
    let mut file = File::create(format!("temp/{uuid}.output"))?;
    file.write_all(t.as_bytes())
}
