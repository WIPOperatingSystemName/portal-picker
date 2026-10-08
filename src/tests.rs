use super::*;
use std::num::NonZeroU32;
fn window(slot: u32, generation: u32) -> CaptureSource {
    CaptureSource::Window(telorgon::shell::WindowId::new(
        NonZeroU32::new(slot).unwrap(),
        NonZeroU32::new(generation).unwrap(),
    ))
}
fn snapshot() -> ScreenCastPortalSnapshot {
    ScreenCastPortalSnapshot {
        pending: Some((1, "Discord".into())),
        source_types: 3,
        sources: vec![
            (
                CaptureSource::Output(telorgon::shell::OutputId::MIN),
                1,
                "Screen 1".into(),
            ),
            (window(1, 1), 2, "Editor".into()),
        ],
        ..Default::default()
    }
}
#[test]
fn multiple_output_identities_are_preserved_by_selection() {
    use selection::Selection;
    let mut value = snapshot();
    let other = CaptureSource::Output(telorgon::shell::OutputId::from_raw(2).unwrap());
    value.sources.push((other, 9, "HDMI".into()));
    let state = Selection {
        request: 1,
        selected: Some((other, 9)),
        ..Default::default()
    };
    assert_eq!(state.current(&value).selected, Some((other, 9)));
    value.sources.pop();
    assert_eq!(state.current(&value).selected, None);
}

#[test]
fn fixed_layout_only_reserves_tabs_for_combined_requests() {
    assert!(!layout::show_tabs(1));
    assert!(!layout::show_tabs(2));
    assert!(layout::show_tabs(3));
    assert_eq!(layout::list_height(1), layout::list_height(2));
    assert_eq!(
        layout::list_height(1) - layout::list_height(3),
        layout::TABS_HEIGHT + layout::GAP
    );
    assert!(layout::list_height(3) > 300.0);
}

#[test]
fn source_type_restrictions_choose_the_tab_even_with_empty_inventory() {
    for (types, tab) in [(1, selection::Tab::Screens), (2, selection::Tab::Windows)] {
        let value = ScreenCastPortalSnapshot {
            source_types: types,
            ..snapshot()
        };
        assert_eq!(selection::Selection::default().current(&value).tab, tab);
    }
    let mut value = snapshot();
    value.sources.clear();
    let mut state = selection::Selection::default().current(&value);
    state.switch(selection::Tab::Windows);
    assert_eq!(state.current(&value).tab, selection::Tab::Windows);
}

#[test]
fn request_and_source_changes_discard_stale_consent() {
    use selection::Selection;
    let mut value = snapshot();
    value.can_remember = true;
    let mut state = Selection::default().current(&value);
    state.choose(window(1, 1), 2, false);
    state.remember = true;
    state.submitted = true;
    value.sources[1].1 += 1;
    let changed = state.current(&value);
    assert!(changed.approved(false).is_empty());
    assert!(!changed.submitted);
    value.pending = Some((2, "Another app".into()));
    let changed = state.current(&value);
    assert!(changed.approved(false).is_empty());
    assert!(!changed.remember);
    assert!(!changed.submitted);
}

#[test]
fn multiple_selection_survives_tabs_but_not_source_replacement() {
    use selection::{Selection, Tab};
    let mut value = snapshot();
    value.multiple = true;
    let mut state = Selection::default().current(&value);
    let screen = value.sources[0].0;
    state.choose(screen, 1, true);
    state.switch(Tab::Windows);
    state.choose(window(1, 1), 2, true);
    assert_eq!(state.approved(true), vec![(screen, 1), (window(1, 1), 2)]);
    assert!(!state.submitted);
    value.sources[1].1 += 1;
    assert!(state.current(&value).approved(true).is_empty());
}

#[test]
fn group_selection_toggles_and_is_bounded() {
    let mut state = selection::Selection::default();
    for i in 1..=9 {
        state.choose(window(i, 1), u64::from(i), true);
    }
    assert_eq!(state.approved(true).len(), 8);
    state.choose(window(1, 1), 1, true);
    state.choose(window(9, 1), 9, true);
    assert_eq!(state.approved(true).len(), 8);
    assert!(state.approved(true).contains(&(window(9, 1), 9)));
}

#[test]
fn removing_persistence_capability_discards_remember_consent() {
    let mut value = snapshot();
    value.can_remember = true;
    let mut state = selection::Selection::default().current(&value);
    state.remember = true;
    value.can_remember = false;
    assert!(!state.current(&value).remember);
}

#[test]
fn disappearing_selection_is_cleared() {
    let mut value = snapshot();
    let mut state = selection::Selection::default().current(&value);
    state.choose(window(1, 1), 2, false);
    value.sources.clear();
    assert!(state.current(&value).approved(false).is_empty());
}
