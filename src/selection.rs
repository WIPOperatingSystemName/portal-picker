//! Selection is presentation state, never authority. All identities retain the host mapping epoch.
use super::{CaptureSource, ScreenCastPortalSnapshot};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Tab {
    #[default]
    Screens,
    Windows,
}
impl Tab {
    pub fn contains(self, source: CaptureSource) -> bool {
        matches!(
            (self, source),
            (
                Self::Screens,
                CaptureSource::Output(_) | CaptureSource::VirtualOutput(_)
            ) | (Self::Windows, CaptureSource::Window(_))
        )
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct Selection {
    pub audio: bool,
    pub request: u64,
    pub tab: Tab,
    pub selected: Option<(CaptureSource, u64)>,
    pub chosen: Vec<(CaptureSource, u64)>,
    pub submitted: bool,
    pub remember: bool,
}
impl Selection {
    pub fn current(&self, snapshot: &ScreenCastPortalSnapshot) -> Self {
        let request = snapshot.pending.as_ref().map_or(0, |p| p.0);
        let mut next = if self.request == request {
            self.clone()
        } else {
            Self {
                request,
                ..Self::default()
            }
        };
        if !snapshot.audio_available { next.audio = false; }
        if !snapshot.can_remember {
            next.remember = false;
        }
        let screens = snapshot.source_types & (1 | 4) != 0;
        let windows = snapshot.source_types & 2 != 0;
        if (!screens && next.tab == Tab::Screens) || (!windows && next.tab == Tab::Windows) {
            next.tab = if screens { Tab::Screens } else { Tab::Windows };
        }
        if next
            .selected
            .is_some_and(|selected| !snapshot.sources.iter().any(|s| (s.0, s.1) == selected))
        {
            next.selected = None;
            next.submitted = false;
        }
        if next
            .chosen
            .iter()
            .any(|selected| !snapshot.sources.iter().any(|s| (s.0, s.1) == *selected))
            || (!snapshot.multiple && !next.chosen.is_empty())
        {
            next.chosen.clear();
            next.submitted = false;
        }
        next
    }
    pub fn choose(&mut self, source: CaptureSource, epoch: u64, multiple: bool) {
        self.selected = Some((source, epoch));
        if multiple {
            if let Some(index) = self.chosen.iter().position(|s| *s == (source, epoch)) {
                self.chosen.remove(index);
            } else if self.chosen.len() < 8 {
                self.chosen.push((source, epoch));
            }
        }
    }
    pub fn approved(&self, multiple: bool) -> Vec<(CaptureSource, u64)> {
        if multiple {
            self.chosen.clone()
        } else {
            self.selected.into_iter().collect()
        }
    }
    pub fn switch(&mut self, tab: Tab) {
        self.tab = tab;
        self.selected = None;
        self.submitted = false;
    }
}
