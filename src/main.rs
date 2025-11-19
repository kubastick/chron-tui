use log::trace;
use rusb::{Context, UsbContext};
use std::time::Duration;

const VENDOR_ID: u16 = 0x3434;
const PRODUCT_ID: u16 = 0xD028;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    trace!("Listing devices");

    let context = Context::new()?;
    let devices = context.devices()?;

    trace!("Looking for mouse receiver");
    let mouse_receiver_device = devices
        .iter()
        .find(|d| {
            let descriptor = d.device_descriptor();
            match descriptor {
                Ok(d) => {
                    let is_mouse_receiver =
                        d.vendor_id() == VENDOR_ID && d.product_id() == PRODUCT_ID;
                    if is_mouse_receiver {
                        trace!("Found mouse receiver device!")
                    }

                    is_mouse_receiver
                }
                Err(_) => false,
            }
        })
        .ok_or(rusb::Error::NotFound)?;

    trace!("Getting mouse receiver handle");
    let mouse_receiver_handle = mouse_receiver_device.open()?;
    let mouse_receiver_dev_desc = mouse_receiver_device.device_descriptor()?;

    trace!("Reading supported languages by mouse receiver device descriptor");
    let timeout = Duration::from_millis(1000);
    let available_languages = mouse_receiver_handle.read_languages(timeout)?;
    for language in available_languages {
        let product_name = mouse_receiver_handle.read_product_string(
            language,
            &mouse_receiver_dev_desc,
            timeout,
        )?;

        trace!("Product name: {product_name}");
    }

    Ok(())
}
