use printpdf::*;

/// The two embedded DejaVu Sans weights used for every generated PDF.
/// DejaVu Sans is bundled at build time (see src-tauri/assets/fonts) so the
/// invoice PDF renders identically on every machine, with no dependency on
/// fonts being installed on the end user's system, and full support for
/// accented French characters.
pub struct InvoiceFonts {
    pub regular: FontId,
    pub bold: FontId,
}

const REGULAR_BYTES: &[u8] = include_bytes!("../../assets/fonts/DejaVuSans.ttf");
const BOLD_BYTES: &[u8] = include_bytes!("../../assets/fonts/DejaVuSans-Bold.ttf");

impl InvoiceFonts {
    pub fn load(doc: &mut PdfDocument) -> Result<Self, String> {
        let mut warnings = Vec::new();
        let regular_font = ParsedFont::from_bytes(REGULAR_BYTES, 0, &mut warnings)
            .ok_or_else(|| "Impossible de charger la police DejaVuSans.ttf".to_string())?;
        let bold_font = ParsedFont::from_bytes(BOLD_BYTES, 0, &mut warnings)
            .ok_or_else(|| "Impossible de charger la police DejaVuSans-Bold.ttf".to_string())?;

        let regular = doc.add_font(&regular_font);
        let bold = doc.add_font(&bold_font);

        Ok(Self { regular, bold })
    }
}
