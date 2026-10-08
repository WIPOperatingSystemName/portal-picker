//! Screen-sharing picker launched by the Telorgon compositor.

use telorgon::app::*;
use telorgon::shell::capture::CaptureSource;
use telorgon::{ScreenCastPortalContext, ScreenCastPortalSnapshot};

mod chooser;
mod preview_images;
mod tile_style;
mod layout;
mod selection;
use chooser::CapturePicker;

asset_catalog! {
    pub mod assets = "assets";
}

#[cfg(test)]
mod tests;

/// App identifiers are untrusted presentation text, even when supplied by the portal frontend.
fn application_label(app_id: &str) -> String {
    let label: String = app_id
        .chars()
        .filter(|c| {
            !c.is_control()
                && !matches!(*c,
        '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
        .take(80)
        .collect();
    if label.trim().is_empty() {
        "An application".into()
    } else {
        label
    }
}

fn main() -> telorgon::Result<()> {
    PortalPickerApplication::new()
        .assets(assets::bundle())
        .renderer(Renderer::Auto)
        .window(|context| {
            let title = context.snapshot().snapshot().pending.as_ref().map_or_else(
                || "Screen sharing".to_owned(),
                |(_, app)| format!("Share with {}", application_label(app)),
            );
            Window::new(title)
                .fixed_size(layout::WIDTH, layout::HEIGHT)
                .content(CapturePicker::new(context))
        })
        .run()
}
