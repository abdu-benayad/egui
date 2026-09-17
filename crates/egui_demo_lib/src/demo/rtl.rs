use egui::text::{LayoutJob, TextWrapping};
use egui::{Align, Align2, Color32, FontFamily, FontId, Id, RichText, TextFormat};

use crate::rtl_font_fixtures::{self, RTL_FONT_FAMILY_NAME};

const EDITOR_ID: &str = "rtl_demo_editor";
const DEFAULT_EDITABLE_TEXT: &str =
    "שלום 12 — hello\nabc אבג def\nمرحبًا بالعالم 123،\nسلام دنیا ۱۲۳،";

fn rtl_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(RTL_FONT_FAMILY_NAME.into()))
}

/// A focused demonstration of the RTL behavior implemented by the current branch.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct RtlDemo {
    editable_text: String,
    spaced_selection_text: String,
    physical_alignment: Align,
    extra_letter_spacing: f32,
    #[cfg_attr(feature = "serde", serde(skip))]
    fonts_installed: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    font_error: Option<String>,
}

impl Default for RtlDemo {
    fn default() -> Self {
        Self {
            editable_text: DEFAULT_EDITABLE_TEXT.to_owned(),
            spaced_selection_text: "אבג".to_owned(),
            physical_alignment: Align::RIGHT,
            extra_letter_spacing: 0.0,
            fonts_installed: false,
            font_error: None,
        }
    }
}

impl RtlDemo {
    fn ensure_fonts(&mut self, ctx: &egui::Context) {
        if self.fonts_installed || self.font_error.is_some() {
            return;
        }
        match rtl_font_fixtures::install(ctx) {
            Ok(()) => self.fonts_installed = true,
            Err(error) => self.font_error = Some(error.to_string()),
        }
    }

    fn static_samples(ui: &mut egui::Ui) {
        ui.heading("Display and fallback");
        ui.label(
            "These samples use checksum-pinned Arabic, Hebrew, and Latin fonts embedded in \
             egui_demo_lib. The family and bytes are identical on native and wasm.",
        );
        egui::Grid::new("rtl_demo_samples")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                for sample in rtl_font_fixtures::POSITIVE_SAMPLES {
                    ui.label(sample.label);
                    ui.label(RichText::new(sample.text).font(rtl_font(20.0)));
                    ui.end_row();
                }
                ui.label("Intentional missing glyph");
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(rtl_font_fixtures::NEGATIVE_MISSING_GLYPH_SAMPLE)
                            .font(rtl_font(20.0)),
                    );
                    ui.weak("negative coverage case; never approve as a success glyph");
                });
                ui.end_row();
            });
    }

    fn layout_samples(&mut self, ui: &mut egui::Ui) {
        ui.heading("Wrapping, styling, spacing, and truncation");
        ui.horizontal(|ui| {
            ui.label("Physical alignment:");
            ui.selectable_value(&mut self.physical_alignment, Align::LEFT, "Left");
            ui.selectable_value(&mut self.physical_alignment, Align::Center, "Center");
            ui.selectable_value(&mut self.physical_alignment, Align::RIGHT, "Right");
        });
        ui.add(
            egui::Slider::new(&mut self.extra_letter_spacing, 0.0..=5.0)
                .text("extra letter spacing"),
        );
        ui.weak(
            "Alignment is physical. Right means the right edge; logical start/end alignment is not implemented.",
        );

        let mut styled = LayoutJob::default();
        styled.wrap = TextWrapping {
            max_width: 310.0,
            ..Default::default()
        };
        styled.halign = self.physical_alignment;
        for (text, color) in [
            ("עברית 12 ", Color32::from_rgb(90, 160, 240)),
            ("مرحبًا 34 ", Color32::from_rgb(230, 150, 60)),
            ("Latin punctuation!?", Color32::PLACEHOLDER),
        ] {
            styled.append(
                text,
                0.0,
                TextFormat {
                    font_id: rtl_font(20.0),
                    color,
                    extra_letter_spacing: self.extra_letter_spacing,
                    ..Default::default()
                },
            );
        }
        ui.add(egui::Label::new(styled).wrap());

        ui.label("Narrow marked text (resize the window to change wrapping):");
        ui.add(
            egui::Label::new(
                RichText::new("بِ بِ بِ — שָׁלוֹם שָׁלוֹם — q\u{1AB0}\u{301}").font(rtl_font(20.0)),
            )
            .wrap(),
        );
        ui.weak(
            "U+1AB0 is intentionally unsupported by the fixtures. Cluster identity, wrapping, and replacement-glyph behavior are validation targets.",
        );

        let mut truncated = LayoutJob::single_section(
            "هذا سطر عربي طويل مع 123 punctuation — טקסט עברי ארוך".to_owned(),
            TextFormat {
                font_id: rtl_font(20.0),
                ..Default::default()
            },
        );
        truncated.wrap = TextWrapping {
            max_width: 310.0,
            max_rows: 1,
            overflow_character: Some('…'),
            ..Default::default()
        };
        truncated.halign = self.physical_alignment;
        ui.label("One-row truncation:");
        ui.add(egui::Label::new(truncated).truncate());
    }

    fn editor(&mut self, ui: &mut egui::Ui) {
        ui.heading("TextEdit interaction");
        ui.label(
            "Try mixed Hebrew plus digits, select the Hebrew word in Latin text, select all across lines, use arrow keys, and compose marked RTL text with your platform IME.",
        );
        let output = egui::TextEdit::multiline(&mut self.editable_text)
            .id(Id::unique(EDITOR_ID))
            .font(rtl_font(19.0))
            .align(Align2::new(self.physical_alignment, Align::TOP))
            .desired_width(460.0)
            .desired_rows(5)
            .show(ui);

        ui.horizontal(|ui| {
            if ui.button("Reset sample").clicked() {
                self.editable_text = DEFAULT_EDITABLE_TEXT.to_owned();
            }
            ui.label("Logical selection:");
            if let Some(range) = output.cursor_range {
                ui.code(range.slice_str(&self.editable_text));
            } else {
                ui.weak("focus the editor and select text");
            }
        });

        ui.label("Spaced RTL selection (drag across the Hebrew text):");
        let spacing = self.extra_letter_spacing.max(2.0);
        let mut spaced_layouter = |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
            let mut job = LayoutJob::single_section(
                text.as_str().to_owned(),
                TextFormat {
                    font_id: rtl_font(20.0),
                    extra_letter_spacing: spacing,
                    ..Default::default()
                },
            );
            job.wrap.max_width = wrap_width;
            ui.fonts_mut(|fonts| fonts.layout_job(job))
        };
        ui.add(
            egui::TextEdit::singleline(&mut self.spaced_selection_text)
                .id(Id::unique("rtl_demo_spaced_editor"))
                .desired_width(180.0)
                .layouter(&mut spaced_layouter),
        );
        ui.collapsing("Exact manual checks", |ui| {
            ui.label("1. In ‘שלום 12’, confirm Hebrew reads RTL while 12 reads LTR.");
            ui.label("2. Replace a line with ‘abc אבג def’; drag across אבג and inspect the highlight.");
            ui.label("3. Select all in the four-line sample; confirm every logical character is selected.");
            ui.label("4. Use left/right arrows at mixed-direction boundaries; record visual caret behavior.");
            ui.label("5. Enter Arabic/Hebrew marks with an IME; record composition, commit, and candidate placement.");
        });
    }

    fn physical_widget_layout(ui: &mut egui::Ui) {
        ui.heading("Physical widget order");
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
            egui::Layout::right_to_left(Align::Center),
            |ui| {
                let _ = ui.button("first in source");
                let _ = ui.button("second in source");
                let _ = ui.button("third in source");
            },
        );
        ui.weak(
            "Layout::right_to_left places these widgets physically. It is not inherited text direction and does not mirror icons or descendants automatically.",
        );
    }
}

impl crate::Demo for RtlDemo {
    fn name(&self) -> &'static str {
        "↔ RTL text"
    }

    fn show(&mut self, ui: &mut egui::Ui, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .default_width(560.0)
            .default_height(720.0)
            .resizable(true)
            .constrain_to(ui.available_rect_before_wrap())
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    use crate::View as _;
                    self.ui(ui);
                });
            });
    }

    fn logic(&mut self, ctx: &egui::Context) {
        // DemoWindows calls this even while the window is closed. Installing
        // before the first visible frame avoids a transient unresolved-family
        // fallback when the user opens the demo.
        self.ensure_fonts(ctx);
    }
}

impl crate::View for RtlDemo {
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.ensure_fonts(ui.ctx());
        if let Some(error) = &self.font_error {
            ui.colored_label(
                Color32::RED,
                format!("RTL fixtures failed to load: {error}"),
            );
            return;
        }
        ui.vertical_centered(|ui| ui.add(crate::egui_github_link_file!()));
        ui.label(
            "This page demonstrates the current branch, including known failures. It does not claim complete RTL support.",
        );
        ui.separator();
        Self::static_samples(ui);
        ui.separator();
        self.layout_samples(ui);
        ui.separator();
        self.editor(ui);
        ui.separator();
        Self::physical_widget_layout(ui);
        ui.separator();
        ui.strong("Known limits on this baseline");
        ui.label("• paragraph direction is first-strong only; there is no explicit override");
        ui.label("• cursor storage and arrow commands are logical and have no bidi affinity");
        ui.label(
            "• mixed-direction selection geometry and Arabic mark identity have known failures",
        );
        ui.label("• AccessKit reports text runs LTR; IME and browser WebGPU are unvalidated");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{CentralPanel, Key, Modifiers, accesskit};
    use egui_kittest::kittest::Queryable as _;
    use egui_kittest::{Harness, HarnessBuilder, SnapshotOptions};

    #[test]
    fn rtl_demo_select_all_keeps_logical_multiline_text() {
        let mut harness = Harness::new_ui_state(
            |ui, demo: &mut RtlDemo| {
                demo.ensure_fonts(ui.ctx());
                CentralPanel::default().show(ui, |ui| demo.editor(ui));
            },
            RtlDemo::default(),
        );
        let editor = harness.get_by_role(accesskit::Role::MultilineTextInput);
        editor.focus();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
        harness.run();

        let state = egui::TextEdit::load_state(&harness.ctx, Id::unique(EDITOR_ID)).unwrap();
        let selected = state.cursor.char_range().unwrap().as_sorted_char_range();
        assert_eq!(selected.start.0, 0);
        assert_eq!(
            selected.end.0,
            harness.state().editable_text.chars().count()
        );
        assert_eq!(
            harness
                .get_by_role(accesskit::Role::MultilineTextInput)
                .value()
                .as_deref(),
            Some(DEFAULT_EDITABLE_TEXT)
        );
    }

    #[test]
    fn rtl_demo_static_snapshot() {
        let mut demo = RtlDemo::default();
        let mut harness = HarnessBuilder::default()
            .with_size(egui::vec2(760.0, 760.0))
            .build_ui(|ui| {
                CentralPanel::default().show(ui, |ui| {
                    demo.ensure_fonts(ui.ctx());
                    RtlDemo::static_samples(ui);
                    ui.separator();
                    demo.layout_samples(ui);
                    ui.separator();
                    RtlDemo::physical_widget_layout(ui);
                });
            });
        harness.run();
        harness.snapshot_options("rtl_demo/static", &SnapshotOptions::default());
    }

    #[test]
    fn rtl_demo_unsupported_combining_mark_is_recorded() {
        let source = "q\u{1AB0}\u{301}";
        let mut fonts = egui::epaint::text::Fonts::new(
            Default::default(),
            rtl_font_fixtures::font_definitions().unwrap(),
        );
        let galley = fonts.with_pixels_per_point(1.0).layout_no_wrap(
            source.to_owned(),
            rtl_font(20.0),
            Color32::WHITE,
        );
        let reconstructed = galley.rows[0].text();

        eprintln!("unsupported-mark source={source:?} row={reconstructed:?}");
        assert_ne!(
            reconstructed, source,
            "this baseline test records the known dropped-mark limitation; update the demo and validation record when the layout bug is fixed"
        );
    }
}
