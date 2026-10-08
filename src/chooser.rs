//! Shell-designed picker contents inside a compositor-decorated Wayland window.
use super::layout::*;
use super::selection::{Selection, Tab};
use super::*;

const PANEL: ColorRgba8 = ColorRgba8::rgba(25, 27, 31, 255);
const PREVIEW: ColorRgba8 = ColorRgba8::rgba(16, 18, 24, 255);
const CARD: ColorRgba8 = ColorRgba8::rgba(39, 44, 57, 255);
const ACCENT: ColorRgba8 = ColorRgba8::rgba(55, 136, 255, 255);
const MUTED: ColorRgba8 = ColorRgba8::rgba(177, 186, 205, 255);

#[telorgon::component]
pub(super) struct CapturePicker {
    #[input]
    ui: ScreenCastPortalContext,
    #[input]
    // A preview model never grants permission; all actions still use the host-issued context.
    preview: Option<Signal<ScreenCastPortalSnapshot>>,
    #[state]
    selection: Selection,
    #[state]
    images: super::preview_images::PreviewImages,
}
impl CapturePicker {
    pub fn new(ui: ScreenCastPortalContext) -> Self {
        Self {
            ui,
            preview: None,
            selection: Selection::default(),
            images: Default::default(),
        }
    }
    fn snapshot_signal(&self) -> &Signal<ScreenCastPortalSnapshot> {
        self.preview.as_ref().unwrap_or_else(|| self.ui.snapshot())
    }
    fn choose(&mut self, request: u64, source: CaptureSource, epoch: u64) {
        let snapshot = self.snapshot_signal().snapshot();
        if snapshot.pending.as_ref().is_some_and(|p| p.0 == request)
            && snapshot
                .sources
                .iter()
                .any(|s| (s.0, s.1) == (source, epoch))
        {
            self.selection = self.selection.current(&snapshot);
            self.selection.choose(source, epoch, snapshot.multiple);
        }
    }
    fn switch(&mut self, tab: Tab) {
        self.selection = self.selection.current(&self.snapshot_signal().snapshot());
        self.selection.switch(tab);
    }
}
impl Component for CapturePicker {
    fn view(&self) -> impl View {
        let snapshot = self.watch(self.snapshot_signal());
        self.render(&snapshot)
    }
}
impl CapturePicker {
    fn render(&self, snapshot: &ScreenCastPortalSnapshot) -> impl View + use<> {
        let state = self.selection.current(snapshot);
        let id = snapshot.pending.as_ref().map_or(0, |pending| pending.0);
        let selected = state.approved(snapshot.multiple);
        let mut body = column().padding(PAD).gap(GAP);
        if show_tabs(snapshot.source_types) {
            let mut tabs = row().height(TABS_HEIGHT).gap(8.0);
            for (tab, title) in [(Tab::Screens, "Monitors"), (Tab::Windows, "Windows")] {
                tabs = tabs.child(
                    button()
                        .accessible_label(title)
                        .child(text(title).size(14.0))
                        .height(TABS_HEIGHT)
                        .background(if state.tab == tab { ACCENT } else { CARD })
                        .on_press(move |this: &mut Self| this.switch(tab)),
                );
            }
            body = body.child(tabs);
        }
        let mut list = column().grid(TILE_WIDTH, ROW_HEIGHT as u16).gap(GAP);
        let mut count = 0;
        for (source, epoch, label) in &snapshot.sources {
            if !state.tab.contains(*source) {
                continue;
            }
            count += 1;
            let (source, epoch) = (*source, *epoch);
            let chosen = selected.contains(&(source, epoch));
            let mut thumbnail = column()
                .height(108.0)
                .background(PREVIEW)
                .corner_radius(3.0);
            if let Some(preview) = snapshot
                .previews
                .iter()
                .find(|p| (p.source, p.epoch) == (source, epoch))
            {
                use telorgon::graphics::render::{
                    ImageAlphaMode, ImageColorEncoding, ImagePixelFormat, ImageResource,
                };
                thumbnail = thumbnail.child(
                    Image::resource(ImageResource {
                        image: self.images.id(source, epoch),
                        content_version: preview.revision,
                        extent: telorgon::SizeI {
                            width: preview.width as i32,
                            height: preview.height as i32,
                        },
                        color_encoding: ImageColorEncoding::Srgb,
                        alpha_mode: ImageAlphaMode::Opaque,
                        pixel_format: ImagePixelFormat::Rgba8,
                        pixels: preview.pixels.clone(),
                    })
                    .width(Dimension::FILL)
                    .height(Dimension::FILL)
                    .accessible_label(format!("Preview of {}", application_label(label))),
                );
            } else {
                thumbnail = thumbnail.child(text("Loading preview…").size(12.0).color(MUTED));
            }
            let title = application_label(label);
            let title = if title.chars().count() > 27 {
                format!("{}…", title.chars().take(26).collect::<String>())
            } else {
                title
            };
            list = list.child(
                stack()
                    .key(format!("source:{id}:{source:?}:{epoch}"))
                    .height(ROW_HEIGHT)
                    .child(
                        button()
                            .accessible_label(title.clone())
                            .child(text(title)
                                .size(12.0)
                                .line_height(16.0)
                                .height(16.0)
                                .color(ColorRgba8::rgba(210, 214, 222, 255)))
                            .height(Dimension::FILL)
                            .width(Dimension::FILL)
                            .padding(Insets::new(111.0, 4.0, 2.0, 4.0))
                            .background(PANEL)
                            .uniform_border(
                                1.0,
                                if chosen {
                                    ACCENT
                                } else {
                                    super::tile_style::BORDER
                                },
                            )
                            .corner_radius(4.0)
                            .inline_style(super::tile_style::style(chosen))
                            .enabled(!state.submitted)
                            .on_press(move |this: &mut Self| this.choose(id, source, epoch)),
                    )
                    .child(column().padding(2.0).child(thumbnail)),
            );
        }
        if count == 0 {
            list = list.child(
                text(if state.tab == Tab::Screens {
                    "No monitors available"
                } else {
                    "No windows available"
                })
                .color(MUTED),
            );
        }
        body = body.child(
            column()
                .key(format!("list:{id}:{:?}:{count}", state.tab))
                .scrollable()
                .child(list),
        );
        let footer = row()
            .height(FOOTER_HEIGHT)
            .padding(PAD)
            .gap(8.0)
            .align_items(Alignment::Center)
            // The host enables audio only for a negotiated client delivery adapter.
            .child(
                column()
                    .width(210.0)
                    .child(switch("Share system audio", state.audio)
                    .enabled(snapshot.audio_available && !state.submitted)
                    .on_change(|this: &mut Self, enabled| {
                        let current = this.snapshot_signal().snapshot();
                        this.selection = this.selection.current(&current);
                        this.selection.audio = enabled && current.audio_available;
                    }))
                    .child(text(if snapshot.audio_available {
                        ""
                    } else {
                        "Unavailable for this application"
                    }).size(11.0).color(MUTED)),
            )
            .child(spacer())
            .child(
                button()
                    .accessible_label("Cancel")
                    .child(text("Cancel").size(14.0))
                    .width(72.0)
                    .height(32.0)
                    .background(CARD)
                    .corner_radius(4.0)
                    .on_press(move |this: &mut Self| {
                        this.ui.deny(id);
                    }),
            )
            .child(
                button()
                    .accessible_label("Share")
                    .child(text("Share").size(14.0))
                    .key(format!("share:{id}"))
                    .width(72.0)
                    .height(32.0)
                    .corner_radius(4.0)
                    .background(ACCENT)
                    .enabled(!selected.is_empty() && !state.submitted)
                    .on_press(move |this: &mut Self| {
                        let snapshot = this.snapshot_signal().snapshot();
                        let current = this.selection.current(&snapshot);
                        if current.request != id
                            || current.submitted
                            || current.approved(snapshot.multiple) != selected
                        {
                            return;
                        }
                        let queued = if current.audio && snapshot.audio_available {
                            this.ui.approve_with_audio(id, &selected)
                        } else if snapshot.multiple {
                            this.ui.approve_many(id, &selected)
                        } else {
                            selected
                                .first()
                                .is_some_and(|&(source, epoch)| this.ui.approve(id, source, epoch))
                        };
                        if queued {
                            this.selection.submitted = true;
                        }
                    }),
            );
        column()
            .background(PANEL)
            .child(body)
            .child(row().height(1.0).background(super::tile_style::BORDER))
            .child(footer)
    }
}

#[cfg(test)]
mod view_tests {
    use super::*;
    use telorgon::input::InputEvent;
    use telorgon::{MonotonicInstant, PointF, SizeI};
    fn fixture(types: u32, count: u32) -> CapturePicker {
        let mut sources: Vec<_> = (1..=count)
            .map(|i| {
                let source = if types == 1 {
                    CaptureSource::Output(telorgon::shell::OutputId::from_raw(i as u64).unwrap())
                } else {
                    CaptureSource::Window(telorgon::shell::WindowId::new(
                        i.try_into().unwrap(),
                        1.try_into().unwrap(),
                    ))
                };
                (
                    source,
                    1,
                    format!(
                        "{} {i}",
                        if types == 1 {
                            "Monitor"
                        } else {
                            "Application window"
                        }
                    ),
                )
            })
            .collect();
        if types == 3 {
            sources.insert(
                0,
                (
                    CaptureSource::Output(telorgon::shell::OutputId::MIN),
                    1,
                    "Monitor 1".into(),
                ),
            );
        }
        let previews = sources
            .iter()
            .map(|(source, epoch, _)| telorgon::PortalSourcePreview {
                source: *source,
                epoch: *epoch,
                revision: 1,
                width: 2,
                height: 1,
                pixels: vec![210, 50, 80, 255, 40, 130, 200, 255].into(),
            })
            .collect();
        let mut picker = CapturePicker::new(ScreenCastPortalContext::default());
        picker.preview = Some(
            Signal::new(ScreenCastPortalSnapshot {
                pending: Some((8, "Discord".into())),
                source_types: types,
                sources,
                previews,
                ..Default::default()
            })
            .0,
        );
        picker
    }
    fn label_node(ui: &telorgon::MountedUi, label: &str) -> Option<telorgon::ui::UiNodeId> {
        ui.semantics.iter().find_map(|(node, semantic)| {
            if semantic.role != telorgon::SemanticRole::Button {
                return None;
            }
            match semantic.name {
                telorgon::SemanticName::Text(text) if ui.string(text) == Some(label) => Some(node),
                _ => None,
            }
        })
    }
    #[test]
    fn dialog_has_no_client_chrome_and_conditional_tabs() {
        for (types, width, height) in [(1, 640, 520), (2, 900, 700), (3, 420, 320)] {
            let mut runtime =
                telorgon::application_host::AppRuntimeCore::from_composed_with_extent(
                    fixture(types, 2),
                    SizeI { width, height },
                )
                .unwrap();
            runtime.queue_input(telorgon::application_host::PlatformInput::Resize(
                telorgon::SizeF {
                    width: width as f32,
                    height: height as f32,
                },
            ));
            runtime.flush_input(MonotonicInstant::ZERO);
            runtime.prepare_frame(MonotonicInstant::ZERO, true).unwrap();
            assert_eq!(label_node(runtime.ui(), "Monitors").is_some(), types == 3);
            assert_eq!(label_node(runtime.ui(), "Windows").is_some(), types == 3);
            assert!(label_node(runtime.ui(), "×").is_none());
            for (node, visual) in runtime.ui().texts.iter() {
                if runtime.ui().string(visual.content) == Some("Share system audio") {
                    assert!(runtime.layout().computed(node).unwrap().border_rect.height <= 18.0);
                }
            }

            let share = label_node(runtime.ui(), "Share").unwrap();
            let rect = runtime.layout().computed(share).unwrap().border_rect;
            assert!(
                (rect.right() - (width as f32 - PAD)).abs() < 1.0,
                "{rect:?}"
            );
            assert!(
                (rect.bottom() - (height as f32 - PAD)).abs() < 1.0,
                "{rect:?}"
            );
            runtime
                .resize(SizeI {
                    width: width + 80,
                    height: height + 100,
                })
                .unwrap();
            runtime
                .prepare_frame(MonotonicInstant::from_nanos(1_000_000), true)
                .unwrap();
            let rect = runtime.layout().computed(share).unwrap().border_rect;
            assert!((rect.right() - (width as f32 + 80.0 - PAD)).abs() < 1.0);
            assert!((rect.bottom() - (height as f32 + 100.0 - PAD)).abs() < 1.0);
            if let Ok(directory) = std::env::var("TELORGON_PORTAL_PREVIEW_DIR") {
                let image = telorgon::HeadlessRuntime::default()
                    .run_composed_once(fixture(types, 8), SizeI { width, height })
                    .unwrap();
                let mut output = format!("P6\n{width} {height}\n255\n").into_bytes();
                output.extend(
                    image
                        .pixels
                        .chunks_exact(4)
                        .flat_map(|pixel| pixel[..3].iter().copied()),
                );
                std::fs::write(format!("{directory}/portal-{types}.ppm"), output).unwrap();
            }
        }
    }
    #[test]
    fn grid_reflows_without_stretching_tiles() {
        let mut runtime = telorgon::application_host::AppRuntimeCore::from_composed_with_extent(
            fixture(2, 8),
            SizeI {
                width: 640,
                height: 520,
            },
        )
        .unwrap();
        for (width, columns) in [(640, 3), (420, 2), (900, 4)] {
            runtime.resize(SizeI { width, height: 520 }).unwrap();
            runtime.prepare_frame(MonotonicInstant::ZERO, true).unwrap();
            let first = label_node(runtime.ui(), "Application window 1").unwrap();
            let wrapped =
                label_node(runtime.ui(), &format!("Application window {}", columns + 1)).unwrap();
            let first = runtime.layout().computed(first).unwrap().border_rect;
            let wrapped = runtime.layout().computed(wrapped).unwrap().border_rect;
            assert_eq!(first.width, TILE_WIDTH as f32);
            assert_eq!(first.height, ROW_HEIGHT);
            assert_eq!(wrapped.x, first.x);
            assert_eq!(wrapped.y, first.y + ROW_HEIGHT + GAP);
        }
    }
    #[test]
    fn changing_window_order_does_not_reuse_preview_resources() {
        let mut picker = fixture(2, 3);
        let mut snapshot = (*picker.preview.as_ref().unwrap().snapshot()).clone();
        for (index, preview) in snapshot.previews.iter_mut().enumerate() {
            preview.revision = 300 - index as u64 * 100;
        }
        let (signal, writer) = Signal::new(snapshot.clone());
        picker.preview = Some(signal);
        let mut runtime = telorgon::application_host::AppRuntimeCore::from_composed_with_extent(
            picker,
            SizeI {
                width: WIDTH,
                height: HEIGHT,
            },
        )
        .unwrap();
        runtime.prepare_frame(MonotonicInstant::ZERO, true).unwrap();
        // Closing the first window moves an older thumbnail into its former slot.
        snapshot.sources.remove(0);
        snapshot.previews.remove(0);
        writer.publish_if_changed(snapshot.clone());
        runtime
            .prepare_frame(MonotonicInstant::from_nanos(1_000_000), true)
            .unwrap();
        snapshot.sources.reverse();
        writer.publish_if_changed(snapshot.clone());
        runtime
            .prepare_frame(MonotonicInstant::from_nanos(2_000_000), true)
            .unwrap();
        // A new incarnation of a source may restart its thumbnail revision.
        snapshot.sources[0].1 += 1;
        let preview = snapshot
            .previews
            .iter_mut()
            .find(|p| p.source == snapshot.sources[0].0)
            .unwrap();
        preview.epoch += 1;
        preview.revision = 1;
        writer.publish_if_changed(snapshot);
        runtime
            .prepare_frame(MonotonicInstant::from_nanos(3_000_000), true)
            .unwrap();
    }
    #[test]
    fn preview_refresh_preserves_label_size_during_hover_changes() {
        let mut picker = fixture(2, 2);
        let mut snapshot = (*picker.preview.as_ref().unwrap().snapshot()).clone();
        let (signal, writer) = Signal::new(snapshot.clone());
        picker.preview = Some(signal);
        let mut runtime = telorgon::application_host::AppRuntimeCore::from_composed_with_extent(
            picker,
            SizeI {
                width: WIDTH,
                height: HEIGHT,
            },
        )
        .unwrap();
        runtime.prepare_frame(MonotonicInstant::ZERO, true).unwrap();
        for tick in 1..=12 {
            // Alternate preview-only updates with entering/leaving the tile.
            if tick % 3 == 0 {
                runtime.queue_input(InputEvent::mouse_moved(PointF {
                    x: if tick % 2 == 0 { 60.0 } else { 630.0 },
                    y: 60.0,
                }));
            }
            snapshot.previews[0].revision += 1;
            writer.publish_if_changed(snapshot.clone());
            let now = MonotonicInstant::from_nanos(tick * 70_000_000);
            runtime.flush_input(now);
            runtime.prepare_frame(now, true).unwrap();
            let ui = runtime.ui();
            let (_, label) = ui
                .texts
                .iter()
                .find(|(_, t)| ui.string(t.content) == Some("Application window 1"))
                .unwrap();
            assert_eq!(
                label.style.size, 12.0,
                "preview update {tick} changed the resolved font size"
            );
            assert_eq!(label.style.line_height, 16.0);
            let source = label_node(ui, "Application window 1").unwrap();
            assert_eq!(ui.box_styles.get(source).unwrap().decoration.background,
                Background::Color(PANEL), "preview update {tick} changed tile color");
        }
    }
    #[test]
    fn audio_toggle_only_changes_when_delivery_is_available() {
        for available in [false, true] {
            let mut picker = fixture(2, 2);
            let mut snapshot = (*picker.preview.as_ref().unwrap().snapshot()).clone();
            snapshot.audio_available = available;
            let (signal, writer) = Signal::new(snapshot);
            picker.preview = Some(signal);
            let mut runtime =
                telorgon::application_host::AppRuntimeCore::from_composed_with_extent(
                    picker,
                    SizeI {
                        width: WIDTH,
                        height: HEIGHT,
                    },
                )
                .unwrap();
            runtime.prepare_frame(MonotonicInstant::ZERO, true).unwrap();
            let node = runtime
                .ui()
                .semantics
                .iter()
                .find_map(|(node, semantic)| match semantic.name {
                    telorgon::SemanticName::Text(text)
                        if semantic.role == telorgon::SemanticRole::Switch
                            && runtime.ui().string(text) == Some("Share system audio") =>
                    {
                        Some(node)
                    }
                    _ => None,
                })
                .unwrap();
            let rect = runtime.layout().computed(node).unwrap().border_rect;
            runtime.queue_input(InputEvent::mouse_moved(PointF {
                x: rect.x + 12.0,
                y: rect.y + rect.height / 2.0,
            }));
            runtime.queue_input(InputEvent::mouse_button(
                telorgon::input::PointerButton::PRIMARY,
                telorgon::input::ButtonState::Pressed,
            ));
            runtime.queue_input(InputEvent::mouse_button(
                telorgon::input::PointerButton::PRIMARY,
                telorgon::input::ButtonState::Released,
            ));
            let now = MonotonicInstant::from_nanos(1_000_000);
            runtime.flush_input(now);
            runtime.prepare_frame(now, true).unwrap();
            let interaction = runtime.ui().interactions.get(node).unwrap();
            assert_eq!(interaction.enabled, available);
            assert_eq!(
                interaction
                    .flags
                    .contains(telorgon::ui::InteractionFlags::CHECKED),
                available
            );
            drop(writer);
        }
    }
    #[test]
    fn clicking_thumbnail_selects_its_source() {
        use telorgon::input::{ButtonState, PointerButton};
        let mut runtime = telorgon::application_host::AppRuntimeCore::from_composed_with_extent(
            fixture(2, 2),
            SizeI {
                width: WIDTH,
                height: HEIGHT,
            },
        )
        .unwrap();
        runtime.prepare_frame(MonotonicInstant::ZERO, true).unwrap();
        let share = label_node(runtime.ui(), "Share").unwrap();
        assert!(!runtime.ui().interactions.get(share).unwrap().enabled);
        runtime.queue_input(InputEvent::mouse_moved(PointF { x: 60.0, y: 60.0 }));
        runtime.queue_input(InputEvent::mouse_button(
            PointerButton::PRIMARY,
            ButtonState::Pressed,
        ));
        runtime.queue_input(InputEvent::mouse_button(
            PointerButton::PRIMARY,
            ButtonState::Released,
        ));
        let now = MonotonicInstant::from_nanos(1_000_000);
        runtime.flush_input(now);
        runtime.prepare_frame(now, true).unwrap();
        let share = label_node(runtime.ui(), "Share").unwrap();
        assert!(runtime.ui().interactions.get(share).unwrap().enabled);
        let source = label_node(runtime.ui(), "Application window 1").unwrap();
        let decoration = runtime.ui().box_styles.get(source).unwrap().decoration;
        assert_eq!(decoration.background, Background::Color(PANEL));
        assert_eq!(decoration.border, telorgon::ui::Border::all(1.0, ACCENT));
    }
    #[test]
    fn long_list_scrolls_without_moving_footer() {
        let mut runtime = telorgon::application_host::AppRuntimeCore::from_composed_with_extent(
            fixture(2, 50),
            SizeI {
                width: WIDTH,
                height: HEIGHT,
            },
        )
        .unwrap();
        runtime.prepare_frame(MonotonicInstant::ZERO, true).unwrap();
        let share = label_node(runtime.ui(), "Share").unwrap();
        let before = runtime.layout().computed(share).unwrap().border_rect;
        let viewport = runtime
            .ui()
            .kinds
            .iter()
            .find_map(|(node, kind)| (*kind == telorgon::NodeKind::Scroll).then_some(node))
            .unwrap();
        runtime.queue_input(InputEvent::mouse_moved(PointF { x: 100.0, y: 90.0 }));
        runtime.queue_input(InputEvent::mouse_scroll(PointF { x: 0.0, y: -200.0 }));
        let now = MonotonicInstant::from_nanos(1_000_000);
        runtime.flush_input(now);
        runtime.prepare_frame(now, true).unwrap();
        assert!(
            runtime
                .ui()
                .layouts
                .get(viewport)
                .is_some_and(|s| s.scroll_offset.y > 0.0),
            "viewport {:?}; children {:?}",
            runtime.layout().computed(viewport),
            runtime
                .ui()
                .nodes
                .children(viewport)
                .map(|n| runtime.layout().computed(n))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            runtime.layout().computed(share).unwrap().border_rect,
            before
        );
        for (delta, expected) in [
            (-100_000.0, 17.0 * (ROW_HEIGHT + GAP) - GAP - list_height(2)),
            (100_000.0, 0.0),
        ] {
            runtime.queue_input(InputEvent::mouse_scroll(PointF { x: 0.0, y: delta }));
            runtime.flush_input(now);
            runtime.prepare_frame(now, true).unwrap();
            assert_eq!(
                runtime
                    .ui()
                    .layouts
                    .get(viewport)
                    .map_or(0.0, |s| s.scroll_offset.y),
                expected
            );
            assert_eq!(
                runtime.layout().computed(share).unwrap().border_rect,
                before
            );
        }
    }
}
