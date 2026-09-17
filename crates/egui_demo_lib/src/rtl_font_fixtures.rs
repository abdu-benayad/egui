//! Reproducible fonts and samples for the RTL demo and regression tests.

use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily, FontInsert, FontPriority, InsertFontFamily};

pub const ARABIC_FONT_NAME: &str = "Noto Sans Arabic RTL fixture";
pub const HEBREW_FONT_NAME: &str = "Noto Sans Hebrew RTL fixture";
pub const LATIN_FONT_NAME: &str = "Noto Sans RTL fixture";
pub const RTL_FONT_FAMILY_NAME: &str = "RTL demo font fallback";

pub const ARABIC_FONT_BYTES: &[u8] = include_bytes!("../data/rtl-fonts/NotoSansArabic-Regular.ttf");
pub const HEBREW_FONT_BYTES: &[u8] = include_bytes!("../data/rtl-fonts/NotoSansHebrew-Regular.ttf");
pub const LATIN_FONT_BYTES: &[u8] = include_bytes!("../data/rtl-fonts/NotoSans-Regular.ttf");

/// A positive rendering sample and the fixture font that must cover every
/// non-whitespace character in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RtlFontSample {
    pub label: &'static str,
    pub text: &'static str,
    pub font_name: &'static str,
    /// Script-specific characters that the named font must cover itself.
    ///
    /// Common characters in `text`, such as spaces and digits, may be supplied
    /// by another fixture in the proportional fallback chain.
    pub required_characters: &'static str,
}

/// Samples shared by the demo and cmap coverage tests.
pub const POSITIVE_SAMPLES: &[RtlFontSample] = &[
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
        required_characters: "مرحبًابالعالم123،ِ",
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

/// This is intentionally absent from both fixtures.
///
/// It exercises the visible missing-glyph/provider fallback path and must never
/// be counted as a positive rendering sample.
pub const NEGATIVE_MISSING_GLYPH_SAMPLE: &str = "漢";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RtlFontFixtureError {
    MissingAsset(&'static str),
    InvalidFont(&'static str),
    MissingPositiveGlyph {
        font_name: &'static str,
        sample: &'static str,
        character: char,
    },
}

impl core::fmt::Display for RtlFontFixtureError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingAsset(name) => write!(f, "RTL font fixture {name:?} is empty"),
            Self::InvalidFont(name) => write!(f, "RTL font fixture {name:?} is not a valid font"),
            Self::MissingPositiveGlyph {
                font_name,
                sample,
                character,
            } => write!(
                f,
                "RTL font fixture {font_name:?} lacks {character:?} required by {sample:?}"
            ),
        }
    }
}

impl std::error::Error for RtlFontFixtureError {}

fn parse_font<'a>(
    name: &'static str,
    bytes: &'a [u8],
) -> Result<ttf_parser::Face<'a>, RtlFontFixtureError> {
    if bytes.is_empty() {
        return Err(RtlFontFixtureError::MissingAsset(name));
    }
    ttf_parser::Face::parse(bytes, 0).map_err(|_| RtlFontFixtureError::InvalidFont(name))
}

fn validate_required_characters(
    face: &ttf_parser::Face<'_>,
    sample: RtlFontSample,
) -> Result<(), RtlFontFixtureError> {
    for character in sample.required_characters.chars() {
        if face.glyph_index(character).is_none() {
            return Err(RtlFontFixtureError::MissingPositiveGlyph {
                font_name: sample.font_name,
                sample: sample.label,
                character,
            });
        }
    }
    Ok(())
}

fn validate_fixture_bytes(
    arabic: &[u8],
    hebrew: &[u8],
    latin: &[u8],
) -> Result<(), RtlFontFixtureError> {
    let arabic = parse_font(ARABIC_FONT_NAME, arabic)?;
    let hebrew = parse_font(HEBREW_FONT_NAME, hebrew)?;
    let latin = parse_font(LATIN_FONT_NAME, latin)?;

    for &sample in POSITIVE_SAMPLES {
        let face = if sample.font_name == ARABIC_FONT_NAME {
            &arabic
        } else if sample.font_name == HEBREW_FONT_NAME {
            &hebrew
        } else {
            &latin
        };
        validate_required_characters(face, sample)?;

        for character in sample
            .text
            .chars()
            .filter(|character| !character.is_whitespace())
        {
            if [&arabic, &hebrew, &latin]
                .iter()
                .all(|face| face.glyph_index(character).is_none())
            {
                return Err(RtlFontFixtureError::MissingPositiveGlyph {
                    font_name: "RTL fixture fallback chain",
                    sample: sample.label,
                    character,
                });
            }
        }
    }

    Ok(())
}

/// Build the deterministic font definitions used by the RTL demo on native and wasm.
///
/// The named [`RTL_FONT_FAMILY_NAME`] fallback order is Noto Sans Arabic, Noto
/// Sans Hebrew, then Noto Sans. Egui's existing proportional family is left
/// unchanged, so installing the fixtures does not alter unrelated demo text.
/// Validation fails loudly before definitions are returned if an embedded file
/// is invalid or lacks a positive-sample glyph.
pub fn font_definitions() -> Result<FontDefinitions, RtlFontFixtureError> {
    validate_fixture_bytes(ARABIC_FONT_BYTES, HEBREW_FONT_BYTES, LATIN_FONT_BYTES)?;

    let mut definitions = FontDefinitions::default();
    definitions.font_data.insert(
        ARABIC_FONT_NAME.to_owned(),
        Arc::new(FontData::from_static(ARABIC_FONT_BYTES)),
    );
    definitions.font_data.insert(
        HEBREW_FONT_NAME.to_owned(),
        Arc::new(FontData::from_static(HEBREW_FONT_BYTES)),
    );
    definitions.font_data.insert(
        LATIN_FONT_NAME.to_owned(),
        Arc::new(FontData::from_static(LATIN_FONT_BYTES)),
    );

    definitions.families.insert(
        FontFamily::Name(RTL_FONT_FAMILY_NAME.into()),
        vec![
            ARABIC_FONT_NAME.to_owned(),
            HEBREW_FONT_NAME.to_owned(),
            LATIN_FONT_NAME.to_owned(),
        ],
    );

    definitions.families.insert(
        FontFamily::Name(ARABIC_FONT_NAME.into()),
        vec![ARABIC_FONT_NAME.to_owned()],
    );
    definitions.families.insert(
        FontFamily::Name(HEBREW_FONT_NAME.into()),
        vec![HEBREW_FONT_NAME.to_owned()],
    );
    definitions.families.insert(
        FontFamily::Name(LATIN_FONT_NAME.into()),
        vec![LATIN_FONT_NAME.to_owned()],
    );

    Ok(definitions)
}

/// Install the shared RTL demo fixtures.
pub fn install(ctx: &egui::Context) -> Result<(), RtlFontFixtureError> {
    validate_fixture_bytes(ARABIC_FONT_BYTES, HEBREW_FONT_BYTES, LATIN_FONT_BYTES)?;

    for (name, bytes) in [
        (ARABIC_FONT_NAME, ARABIC_FONT_BYTES),
        (HEBREW_FONT_NAME, HEBREW_FONT_BYTES),
        (LATIN_FONT_NAME, LATIN_FONT_BYTES),
    ] {
        ctx.add_font(FontInsert::new(
            name,
            FontData::from_static(bytes),
            vec![
                InsertFontFamily {
                    family: FontFamily::Name(RTL_FONT_FAMILY_NAME.into()),
                    priority: FontPriority::Lowest,
                },
                InsertFontFamily {
                    family: FontFamily::Name(name.into()),
                    priority: FontPriority::Lowest,
                },
            ],
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use sha2::{Digest as _, Sha256};

    use super::*;

    fn sha256(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    #[test]
    fn rtl_font_fixtures_integrity_and_fallback_order() {
        assert_eq!(
            sha256(ARABIC_FONT_BYTES),
            "bdff3e5659d67e67def05b33f749683b9376ae819d65d3dd62ac4640b3aaef48"
        );
        assert_eq!(
            sha256(HEBREW_FONT_BYTES),
            "cdefaf8efd47045f6820928eba84db5bed7557539328952b5f828315485e02ee"
        );
        assert_eq!(
            sha256(LATIN_FONT_BYTES),
            "478c558ea716033cd60c03438f628dfa75694dcf6b5f6d505a2f05fd2b4f3823"
        );
        assert_eq!(
            sha256(include_bytes!("../data/rtl-fonts/OFL-NotoSansArabic.txt")),
            "a7a5a25eb188bf1cd96982030d53e23c33485c69b1044a562254226857ee13af"
        );
        assert_eq!(
            sha256(include_bytes!("../data/rtl-fonts/OFL-NotoSansHebrew.txt")),
            "9b9fe028b5ba74d231659a1bbaf0ed09b11e759d1ca6a070999e16d151616b47"
        );
        assert_eq!(
            sha256(include_bytes!("../data/rtl-fonts/OFL-NotoSans.txt")),
            "cee9892f9f0cc8fe882c9e9537ee6a89621d86ee7ceaf70b02e2b2b1c25c061a"
        );

        let definitions = font_definitions().unwrap();
        let rtl_fallback = &definitions.families[&FontFamily::Name(RTL_FONT_FAMILY_NAME.into())];
        assert_eq!(
            rtl_fallback.as_slice(),
            [
                ARABIC_FONT_NAME.to_owned(),
                HEBREW_FONT_NAME.to_owned(),
                LATIN_FONT_NAME.to_owned()
            ]
        );
    }

    #[test]
    fn rtl_font_fixtures_cover_positive_and_reject_negative_samples() {
        validate_fixture_bytes(ARABIC_FONT_BYTES, HEBREW_FONT_BYTES, LATIN_FONT_BYTES).unwrap();

        let arabic = parse_font(ARABIC_FONT_NAME, ARABIC_FONT_BYTES).unwrap();
        let hebrew = parse_font(HEBREW_FONT_NAME, HEBREW_FONT_BYTES).unwrap();
        let latin = parse_font(LATIN_FONT_NAME, LATIN_FONT_BYTES).unwrap();
        for character in NEGATIVE_MISSING_GLYPH_SAMPLE.chars() {
            assert!(arabic.glyph_index(character).is_none());
            assert!(hebrew.glyph_index(character).is_none());
            assert!(latin.glyph_index(character).is_none());
        }
    }

    #[test]
    fn rtl_font_fixtures_report_missing_and_corrupt_assets() {
        assert_eq!(
            validate_fixture_bytes(&[], HEBREW_FONT_BYTES, LATIN_FONT_BYTES),
            Err(RtlFontFixtureError::MissingAsset(ARABIC_FONT_NAME))
        );
        assert_eq!(
            validate_fixture_bytes(&[0, 1, 2, 3], HEBREW_FONT_BYTES, LATIN_FONT_BYTES),
            Err(RtlFontFixtureError::InvalidFont(ARABIC_FONT_NAME))
        );
        assert_eq!(
            validate_fixture_bytes(ARABIC_FONT_BYTES, &[], LATIN_FONT_BYTES),
            Err(RtlFontFixtureError::MissingAsset(HEBREW_FONT_NAME))
        );
        assert_eq!(
            validate_fixture_bytes(ARABIC_FONT_BYTES, &[0, 1, 2, 3], LATIN_FONT_BYTES),
            Err(RtlFontFixtureError::InvalidFont(HEBREW_FONT_NAME))
        );
        assert_eq!(
            validate_fixture_bytes(ARABIC_FONT_BYTES, HEBREW_FONT_BYTES, &[]),
            Err(RtlFontFixtureError::MissingAsset(LATIN_FONT_NAME))
        );
        assert_eq!(
            validate_fixture_bytes(ARABIC_FONT_BYTES, HEBREW_FONT_BYTES, &[0, 1, 2, 3]),
            Err(RtlFontFixtureError::InvalidFont(LATIN_FONT_NAME))
        );
    }

    #[test]
    fn rtl_font_fixtures_install_preserves_host_fonts_and_families() {
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
        let mut output = ctx.run_ui(Default::default(), |_| {});
        output.textures_delta.clear();

        install(&ctx).unwrap();
        // This second call happens before pending fonts become active.
        install(&ctx).unwrap();
        let mut output = ctx.run_ui(Default::default(), |_| {});
        output.textures_delta.clear();

        // Installing again after activation is also harmless.
        install(&ctx).unwrap();
        let mut output = ctx.run_ui(Default::default(), |_| {});
        output.textures_delta.clear();

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
            assert_eq!(
                definitions.families[&FontFamily::Name(RTL_FONT_FAMILY_NAME.into())],
                [
                    ARABIC_FONT_NAME.to_owned(),
                    HEBREW_FONT_NAME.to_owned(),
                    LATIN_FONT_NAME.to_owned(),
                ]
            );
            assert_eq!(
                definitions.families[&FontFamily::Name(RTL_FONT_FAMILY_NAME.into())]
                    .iter()
                    .filter(|name| name.as_str() == ARABIC_FONT_NAME)
                    .count(),
                1
            );
        });
    }
}
