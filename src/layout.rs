//! Fixed logical dimensions for the picker client window.
pub(super) const WIDTH: i32 = 640;
pub(super) const HEIGHT: i32 = 520;
pub(super) const PAD: f32 = 8.0;
pub(super) const GAP: f32 = 8.0;
pub(super) const TABS_HEIGHT: f32 = 28.0;
pub(super) const FOOTER_HEIGHT: f32 = 48.0;
pub(super) const TILE_WIDTH: u16 = 196;
pub(super) const ROW_HEIGHT: f32 = 136.0;

pub(super) fn show_tabs(types: u32) -> bool {
    types & 2 != 0 && types & (1 | 4) != 0
}
#[cfg(test)]
pub(super) fn list_height(types: u32) -> f32 {
    HEIGHT as f32
        - PAD * 2.0
        - FOOTER_HEIGHT
        - 1.0
        - if show_tabs(types) {
            TABS_HEIGHT + GAP
        } else {
            0.0
        }
}
