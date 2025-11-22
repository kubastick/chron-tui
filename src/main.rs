mod mouse_device;
mod tui_app;

use crate::mouse_device::{ MouseDevice};
use crate::tui_app::App;
use hidapi::HidApi;
use log::trace;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    trace!("Initializing HID API");
    let api = HidApi::new()?;
    api.set_open_exclusive(false);

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
    let mut terminal = ratatui::init();
    let app_result = App {
        device_name: product_name,
        dpi_config: dpi_cfg,
        current_dpi_index: 0,
        exit: false,
    }
    .run(&mut terminal);
    ratatui::restore();
    app_result?;

    Ok(())
}
