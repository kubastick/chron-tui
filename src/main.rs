mod mouse_device;

use crate::mouse_device::{DpiConfig, MouseDevice};
use hidapi::HidApi;
use log::trace;
use ratatui::crossterm::event;
use ratatui::crossterm::event::Event;
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::Stylize;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::{DefaultTerminal, Frame};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    trace!("Initializing HID API");
    let api = HidApi::new()?;

    trace!("Initializing mouse receiver device");
    let vendor_id = 0x3434;
    let product_id = 0xd028;
    let mouse_hid_d = api.open(vendor_id, product_id)?;
    mouse_hid_d.set_blocking_mode(false)?;
    let product_name = mouse_hid_d.get_product_string()?.unwrap_or_default();
    trace!("Opened mouse device! ({product_name})");

    let mouse_d = MouseDevice::new(mouse_hid_d);
    let dpi_cfg = mouse_d.get_dpi_config()?;

    trace!("DPI config: {:?}", dpi_cfg);

    trace!("Initializing terminal user interface");
    color_eyre::install()?;
    let terminal = ratatui::init();
    let tui_result = run_tui(terminal, product_name, dpi_cfg);
    ratatui::restore();

    tui_result
}

fn run_tui(
    mut terminal: DefaultTerminal,
    device_name: String,
    dpi_config: DpiConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| render(f, device_name.clone(), dpi_config.clone()))?;
        if matches!(event::read()?, Event::Key(_)) {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame, device_name: String, dpi_config: DpiConfig) {
    let mut dpi_list: Vec<ListItem> = Vec::with_capacity(dpi_config.presets.len());
    for i in 0..dpi_config.presets.len() {
        let natural_index = i + 1;
        let dpi_value = &dpi_config.presets[i];
        let dpi_preset_active = i == dpi_config.current_index.into();
        let dpi_preset_active_symbol = match dpi_preset_active {
            true => "*".to_owned(),
            false => " ".to_owned(),
        };

        let list_str = format!("#{natural_index} [{dpi_preset_active_symbol}] {dpi_value}");
        let list_item = ListItem::new(list_str).style(match dpi_preset_active {
            true => Style::new().bold(),
            false => Style::default(),
        });
        dpi_list.push(list_item);
    }

    let vertical = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
    let horizontal = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(2)]);
    let [title_bar, main_area] = vertical.areas(frame.area());
    let [left, right] = horizontal.areas(main_area);

    frame.render_widget(
        Block::new()
            .borders(Borders::TOP)
            .style(Style::new().cyan())
            .title(format!("chron_tui - {device_name}")),
        title_bar,
    );

    let dpi_list = List::new(dpi_list);
    let dpi_preset_block = Block::bordered()
        .style(Style::new().green())
        .title("DPI Presets");
    frame.render_widget(dpi_preset_block.clone(), left);
    frame.render_widget(dpi_list, dpi_preset_block.inner(left));

    frame.render_widget(
        Block::bordered()
            .style(Style::new().yellow())
            .title("Polling Rate"),
        right,
    );
}
