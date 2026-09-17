//! Reproducible fonts and samples for the RTL demo.

use egui::{FontData, FontFamily, FontInsert, FontPriority, Id, InsertFontFamily};

pub(crate) const ARABIC_FONT_NAME: &str = "Noto Sans Arabic RTL fixture";
pub(crate) const HEBREW_FONT_NAME: &str = "Noto Sans Hebrew RTL fixture";
pub(crate) const LATIN_FONT_NAME: &str = "Ubuntu Light RTL fixture";
pub(crate) const RTL_FONT_FAMILY_NAME: &str = "RTL demo font fallback";

const ARABIC_FONT_BYTES: &[u8] = include_bytes!("../data/rtl-fonts/NotoSansArabic-Regular.ttf");
const HEBREW_FONT_BYTES: &[u8] = include_bytes!("../data/rtl-fonts/NotoSansHebrew-Regular.ttf");
const LATIN_FONT_BYTES: &[u8] = epaint_default_fonts::UBUNTU_LIGHT;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RtlFontSample {
    pub label: &'static str,
    pub text: &'static str,
    pub font_name: &'static str,
    /// Script-specific characters that the named font must cover itself.
    ///
    /// Common characters in `text`, such as spaces and digits, may be supplied
    /// by another fixture in the named fallback family.
    pub required_characters: &'static str,
}

/// Samples shared by the demo and fixture coverage tests.
pub(crate) const POSITIVE_SAMPLES: &[RtlFontSample] = &[
    RtlFontSample {
        label: "Latin, digits, and punctuation",
        text: "egui 123,.-!?",
        font_name: LATIN_FONT_NAME,
        required_characters: "egui123,.-!?",
    },
    RtlFontSample {
        label: "Arabic with a combining mark",
        text: "مرحبًا بالعالم 123، بِ",
        font_name: ARABIC_FONT_NAME,
        required_characters: "مرحبًابالعالم،ِ",
    },
    RtlFontSample {
        label: "Persian",
        text: "سلام دنیا ۱۲۳،",
        font_name: ARABIC_FONT_NAME,
        required_characters: "سلامدنیا۱۲۳،",
    },
    RtlFontSample {
        label: "Urdu",
        text: "پاکستان میں اردو ۱۲۳؟",
        font_name: ARABIC_FONT_NAME,
        required_characters: "پاکستانمیںاردو۱۲۳؟",
    },
    RtlFontSample {
        label: "Hebrew with points",
        text: "שָׁלוֹם עולם 123!",
        font_name: HEBREW_FONT_NAME,
        required_characters: "שָׁלוֹםעולם",
    },
];

/// This is intentionally absent from all three fonts.
///
/// It exercises the visible missing-glyph/provider fallback path and must never
/// be counted as a positive rendering sample.
pub(crate) const NEGATIVE_MISSING_GLYPH_SAMPLE: &str = "漢";

#[derive(Clone)]
struct RtlFixturesInstalled;

fn family_insert(family: FontFamily) -> InsertFontFamily {
    InsertFontFamily {
        family,
        priority: FontPriority::Lowest,
    }
}

/// Add the RTL demo fonts without replacing any fonts or families already
/// configured by the host application.
///
/// The context marker makes repeated calls idempotent, including multiple calls
/// before the next egui pass activates pending fonts.
pub(crate) fn install(ctx: &egui::Context) {
    let marker = Id::unique("egui_demo_lib::rtl_font_fixtures::installed");
    let already_installed = ctx.data_mut(|data| {
        if data.get_temp::<RtlFixturesInstalled>(marker).is_some() {
            true
        } else {
            data.insert_temp(marker, RtlFixturesInstalled);
            false
        }
    });
    if already_installed {
        return;
    }

    for (name, bytes) in [
        (ARABIC_FONT_NAME, ARABIC_FONT_BYTES),
        (HEBREW_FONT_NAME, HEBREW_FONT_BYTES),
        (LATIN_FONT_NAME, LATIN_FONT_BYTES),
    ] {
        ctx.add_font(FontInsert::new(
            name,
            FontData::from_static(bytes),
            vec![
                family_insert(FontFamily::Name(RTL_FONT_FAMILY_NAME.into())),
                family_insert(FontFamily::Name(name.into())),
            ],
        ));
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use egui::FontDefinitions;

    use super::*;

    fn face<'a>(name: &str, bytes: &'a [u8]) -> ttf_parser::Face<'a> {
        ttf_parser::Face::parse(bytes, 0)
            .unwrap_or_else(|error| panic!("invalid RTL demo font {name:?}: {error:?}"))
    }

    #[test]
    fn rtl_font_fixtures_cover_positive_and_reject_negative_samples() {
        let arabic = face(ARABIC_FONT_NAME, ARABIC_FONT_BYTES);
        let hebrew = face(HEBREW_FONT_NAME, HEBREW_FONT_BYTES);
        let latin = face(LATIN_FONT_NAME, LATIN_FONT_BYTES);

        for &sample in POSITIVE_SAMPLES {
            let required_face = if sample.font_name == ARABIC_FONT_NAME {
                &arabic
            } else if sample.font_name == HEBREW_FONT_NAME {
                &hebrew
            } else {
                &latin
            };
            for character in sample.required_characters.chars() {
                assert!(
                    required_face.glyph_index(character).is_some(),
                    "{:?} lacks {character:?} required by {:?}",
                    sample.font_name,
                    sample.label
                );
            }

            for character in sample.text.chars().filter(|c| !c.is_whitespace()) {
                assert!(
                    [&arabic, &hebrew, &latin]
                        .iter()
                        .any(|font| font.glyph_index(character).is_some()),
                    "RTL fallback family lacks {character:?} from {:?}",
                    sample.label
                );
            }
        }

        for character in NEGATIVE_MISSING_GLYPH_SAMPLE.chars() {
            assert!(arabic.glyph_index(character).is_none());
            assert!(hebrew.glyph_index(character).is_none());
            assert!(latin.glyph_index(character).is_none());
        }
    }

    #[test]
    fn install_is_additive_and_idempotent_before_and_after_a_pass() {
        fn run_empty(ctx: &egui::Context) {
            let mut output = ctx.run_ui(Default::default(), |_| {});
            output.textures_delta.clear();
        }

        const HOST_FONT_NAME: &str = "host custom font";
        let host_family = FontFamily::Name("host custom family".into());

        let mut host_definitions = FontDefinitions::default();
        let proportional_before = host_definitions.families[&FontFamily::Proportional].clone();
        host_definitions.font_data.insert(
            HOST_FONT_NAME.to_owned(),
            Arc::new(FontData::from_static(LATIN_FONT_BYTES)),
        );
        host_definitions
            .families
            .insert(host_family.clone(), vec![HOST_FONT_NAME.to_owned()]);

        let ctx = egui::Context::default();
        ctx.set_fonts(host_definitions);
        run_empty(&ctx);

        install(&ctx);
        install(&ctx); // same pass: pending additions must not be duplicated
        run_empty(&ctx);

        install(&ctx); // active fonts must not be duplicated either
        run_empty(&ctx);

        ctx.fonts(|fonts| {
            let definitions = fonts.definitions();
            assert!(definitions.font_data.contains_key(HOST_FONT_NAME));
            assert_eq!(
                definitions.families[&host_family],
                [HOST_FONT_NAME.to_owned()]
            );
            assert_eq!(
                definitions.families[&FontFamily::Proportional],
                proportional_before
            );

            let rtl_family = &definitions.families[&FontFamily::Name(RTL_FONT_FAMILY_NAME.into())];
            assert_eq!(
                rtl_family,
                &[
                    ARABIC_FONT_NAME.to_owned(),
                    HEBREW_FONT_NAME.to_owned(),
                    LATIN_FONT_NAME.to_owned(),
                ]
            );
        });
    }
}
