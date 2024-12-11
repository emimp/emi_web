use std::{fs::{self, File}, io::{self, Write}, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    backend::TestBackend, buffer::Buffer, layout::Rect, style::Stylize, symbols::border, text::{Line, Text}, widgets::{Block, Paragraph, Widget}, DefaultTerminal, Frame, Terminal
};
// ANCHOR_END: imports

pub fn init() -> io::Result<()> {
    let contents = fs::read_to_string("resize.txt").unwrap().split(',')
        .map(|s| s.parse().expect("Failed to parse"))
        .collect::<Vec<u16>>();
    let width = contents[0]/25;
    let height = contents[1]/50;
    
    
    let backend = TestBackend::new(width,height); // Simulates a terminal with 80x24 size
    let terminal = ratatui::Terminal::new(backend).unwrap();
    let app_result = App::default().run(terminal);
    ratatui::restore();
    app_result
}

// ANCHOR: app
#[derive(Debug, Default)]
pub struct App {
    counter: u8,
    exit: bool,
}
// ANCHOR_END: app

// ANCHOR: impl App
impl App {
    /// runs the application's main loop until the user quits
    pub fn run<T: ratatui::backend::Backend>(&mut self, mut terminal: Terminal<T>) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            // self.handle_events()?;
            self.handle_web_key_event();
            let seconds = Duration::from_millis(10);
            std::thread::sleep(seconds);
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    /// updates the application's state based on user input
    fn handle_web_key_event(&mut self) {
        let contents = fs::read_to_string("key_log.txt").unwrap();
        let key = contents.as_str();
        match key {
            "ArrowLeft" => self.decrement_counter(),
            "ArrowRight" => self.increment_counter(),
            _ => {}
        }
        let mut file = File::create("key_log.txt").expect("Failed to create or open the file");
        file.write_all( "".as_bytes()).expect("Failed to write to file");
    }
    // ANCHOR: handle_key_event fn
    // fn handle_key_event(&mut self, key_event: KeyEvent) {
    //     match key_event.code {
    //         KeyCode::Char('q') => self.exit(),
    //         KeyCode::Left => self.decrement_counter(),
    //         KeyCode::Right => self.increment_counter(),
    //         _ => {}
    //     }
    // }
    // ANCHOR_END: handle_key_event fn

    fn exit(&mut self) {
        self.exit = true;
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
// ANCHOR_END: impl App

// ANCHOR: impl Widget
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

        frame_to_file(buf).expect("couldnt frame to file");
        let frame = format!("{:?}",buf);
    }
}

fn frame_to_file (buf:& mut Buffer) -> io::Result<()> {
    let t = format!("{:?}",buf);
    let mut file = File::create("output.txt").expect("msg");
    file.write_all(t.as_bytes())
}