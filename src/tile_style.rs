use std::{collections::BTreeMap, sync::Arc};
use telorgon::app::*;
use telorgon::theme::{
    CompiledComponentStyle, CompiledSlotStyle, CompiledStateStyle, InteractionState,
};
use telorgon::ui::{
    ComponentStyleId, InteractionFlags, StylePropertyPatch, StyleSlotId, ThemeDomainId,
};
pub(super) const BORDER: ColorRgba8 = ColorRgba8::rgba(57, 61, 68, 255);
pub(super) fn style(selected: bool) -> Arc<CompiledComponentStyle> {
    let slot = StyleSlotId::named("root");
    let accent = ColorRgba8::rgba(55, 136, 255, 255);
    let patch = StylePropertyPatch {
        background: Some(Background::Color(ColorRgba8::rgba(25, 27, 31, 255))),
        border: Some(telorgon::ui::Border::all(
            1.0,
            if selected { accent } else { BORDER },
        )),
        ..Default::default()
    };
    Arc::new(CompiledComponentStyle {
        id: ComponentStyleId::named(ThemeDomainId::SHELL, "portal", "source"),
        slots: BTreeMap::from([
            (
                slot,
                CompiledSlotStyle {
                    patch,
                    font_family: None,
                },
            ),
        ]),
        variants: Default::default(),
        states: BTreeMap::from([(
            InteractionState::Hovered,
            CompiledStateStyle {
                slots: BTreeMap::from([(
                    slot,
                    CompiledSlotStyle {
                        patch: StylePropertyPatch {
                            border_color: Some(if selected {
                                accent
                            } else {
                                ColorRgba8::rgba(105, 112, 124, 255)
                            }),
                            ..Default::default()
                        },
                        font_family: None,
                    },
                )]),
                transition: None,
            },
        )]),
        state_precedence: vec![InteractionState::Hovered],
        relevant_states: InteractionFlags::HOVERED,
        transition: Default::default(),
        controlled_slots: BTreeMap::from([(slot, patch)]),
        controlled_font_families: Default::default(),
    })
}
