use crate::mouse_device::DpiConfig;
use ratatui::crossterm::event;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Block, List, ListItem};
use std::io;

#[derive(Debug)]
pub struct App {
    pub device_name: String,
    pub dpi_config: DpiConfig,
    pub current_dpi_index: i16,
    pub exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => {
                self.exit = true;
            }
            KeyCode::Up => {
                self.current_dpi_index -= 1;
                self.current_dpi_index = self.current_dpi_index.max(0);
            }
            KeyCode::Down => {
                self.current_dpi_index += 1;
                let max_index = self.dpi_config.presets.len() as i16 - 1;
                self.current_dpi_index = self.current_dpi_index.min(max_index);
            }
            _ => {}
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(format!(" chron_tui - {0} ", self.device_name).bold());
        let instructions = Line::from(vec![
            " Choose ".into(),
            "<Up/Down>".blue().bold(),
            " Select ".into(),
            "<Enter>".blue().bold(),
            " Edit ".into(),
            "<E>".blue().bold(),
            " Switch tab ".into(),
            "<Tab>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.left_aligned())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);
        block.clone().render(area, buf);

        let mut dpi_list: Vec<ListItem> = Vec::with_capacity(self.dpi_config.presets.len());
        for i in 0..self.dpi_config.presets.len() {
            let natural_index = i + 1;
            let dpi_value = &self.dpi_config.presets[i];
            let dpi_highlighted = i == self.current_dpi_index as usize;
            let dpi_preset_active = i == self.dpi_config.current_index.into();
            let dpi_preset_active_symbol = match dpi_preset_active {
                true => "*".to_owned(),
                false => " ".to_owned(),
            };
            let dpi_highlighted_symbol = match dpi_highlighted {
                true => "-> ".to_owned(),
                false => " ".to_owned(),
            };

            let list_str = format!("{dpi_highlighted_symbol}#{natural_index} [{dpi_preset_active_symbol}] {dpi_value}");
            let list_item_style = match dpi_highlighted {
                true => match dpi_preset_active {
                    true => Style::new().bold().bg(Color::Green).fg(Color::White),
                    false => Style::default().bg(Color::Green).fg(Color::White),
                },
                false => match dpi_preset_active {
                    true => Style::new().bold(),
                    false => Style::default(),
                },
            };

            let list_item = ListItem::new(list_str).style(list_item_style);
            dpi_list.push(list_item);
        }

        let horizontal = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(2)]);
        let [left, right] = horizontal.areas(block.inner(area));

        let dpi_preset_block = Block::bordered()
            .style(Style::new().green())
            .title("DPI Presets");
        dpi_preset_block.clone().render(left, buf);

        let dpi_list = List::new(dpi_list);

        ratatui::prelude::Widget::render(dpi_list, dpi_preset_block.inner(left), buf);

        Block::bordered()
            .style(Style::new().yellow())
            .title(" Polling Rate ")
            .render(right, buf);
    }
}
