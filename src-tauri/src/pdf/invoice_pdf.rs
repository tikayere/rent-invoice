use printpdf::*;

use crate::models::{Invoice, Settings, Tenant};
use crate::pdf::fonts::InvoiceFonts;

const PAGE_WIDTH: f32 = 210.0;
const PAGE_HEIGHT: f32 = 297.0;
const MARGIN: f32 = 15.0;
const CONTENT_RIGHT: f32 = PAGE_WIDTH - MARGIN;

const DARK: (f32, f32, f32) = (0.117, 0.161, 0.231); // slate-900
const GREY: (f32, f32, f32) = (0.392, 0.455, 0.545); // slate-500
const LIGHT_GREY: (f32, f32, f32) = (0.945, 0.957, 0.976); // slate-100
const WHITE: (f32, f32, f32) = (1.0, 1.0, 1.0);

/// Selectable invoice layouts. Persisted on `Settings.invoice_template` as
/// its lowercase name; unrecognised values fall back to `Classic`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvoiceTemplate {
    Classic,
    Modern,
    Minimal,
}

impl InvoiceTemplate {
    pub fn from_key(key: &str) -> Self {
        match key {
            "modern" => InvoiceTemplate::Modern,
            "minimal" => InvoiceTemplate::Minimal,
            _ => InvoiceTemplate::Classic,
        }
    }
}

/// The colours and accents that vary between [`InvoiceTemplate`]s. Layout
/// (positions, sizes, spacing) stays identical across templates so that
/// swapping templates can never break pagination.
struct Palette {
    /// Accent colour used for the title and the header divider.
    brand: (f32, f32, f32),
    /// Weight (in pt) of the divider line under the header block.
    header_rule_weight: f32,
    /// Charges table header row background/text colours.
    table_header_bg: (f32, f32, f32),
    table_header_fg: (f32, f32, f32),
    /// Zebra-striping colour for odd charge rows; `None` disables striping.
    zebra: Option<(f32, f32, f32)>,
    /// Draws a hairline under the charges table header row. Needed when the
    /// header background is too close to the page background to read as a
    /// distinct band on its own (the Minimal template).
    table_header_border: bool,
}

impl Palette {
    fn for_template(template: InvoiceTemplate) -> Self {
        match template {
            InvoiceTemplate::Classic => Palette {
                brand: (0.145, 0.388, 0.922), // #2563eb
                header_rule_weight: 1.2,
                table_header_bg: DARK,
                table_header_fg: WHITE,
                zebra: Some(LIGHT_GREY),
                table_header_border: false,
            },
            InvoiceTemplate::Modern => Palette {
                brand: (0.024, 0.467, 0.435), // #0c7768 teal
                header_rule_weight: 2.6,
                table_header_bg: (0.024, 0.467, 0.435),
                table_header_fg: WHITE,
                zebra: Some((0.902, 0.961, 0.949)), // teal-tinted slate-100
                table_header_border: false,
            },
            InvoiceTemplate::Minimal => Palette {
                brand: DARK, // monochrome: no accent colour
                header_rule_weight: 0.5,
                table_header_bg: WHITE,
                table_header_fg: DARK,
                zebra: None,
                table_header_border: true,
            },
        }
    }
}

/// Everything needed to render one invoice to a byte buffer. Kept as owned
/// data (rather than borrowing straight from SQLite rows) so PDF generation
/// has no dependency on the database connection being held open.
pub struct InvoicePdfData<'a> {
    pub settings: &'a Settings,
    pub tenant: &'a Tenant,
    pub invoice: &'a Invoice,
}

/// Renders a complete, printable A4 rent invoice and returns the raw PDF bytes.
pub fn render_invoice_pdf(data: &InvoicePdfData) -> Result<Vec<u8>, String> {
    let mut doc = PdfDocument::new(&format!("Facture {}", data.invoice.invoice_number));
    let fonts = InvoiceFonts::load(&mut doc)?;
    let palette = Palette::for_template(InvoiceTemplate::from_key(&data.settings.invoice_template));

    let mut ops: Vec<Op> = Vec::new();
    let mut cursor = PdfCursor { y: PAGE_HEIGHT - MARGIN };

    render_header(&mut ops, &mut doc, &fonts, data, &mut cursor, &palette);
    render_title_and_meta(&mut ops, &fonts, data, &mut cursor, &palette);
    render_tenant_and_property(&mut ops, &fonts, data, &mut cursor);
    render_charges_table(&mut ops, &fonts, data, &mut cursor, &palette);
    render_totals(&mut ops, &fonts, data, &mut cursor);
    render_payment_info(&mut ops, &fonts, data, &mut cursor);
    render_signature(&mut ops, &mut doc, &fonts, data);
    render_footer(&mut ops, &fonts, data);

    let page = PdfPage::new(Mm(PAGE_WIDTH), Mm(PAGE_HEIGHT), ops);
    let save_options = PdfSaveOptions {
        subset_fonts: true,
        ..Default::default()
    };
    let mut warnings = Vec::new();
    let bytes = doc.with_pages(vec![page]).save(&save_options, &mut warnings);
    Ok(bytes)
}

/// Tracks the current vertical writing position (in mm from the bottom of
/// the page, matching printpdf's coordinate system) as content is laid out.
struct PdfCursor {
    y: f32,
}

// ---------------------------------------------------------------------
// Low level drawing helpers
// ---------------------------------------------------------------------

fn rgb(c: (f32, f32, f32)) -> Color {
    Color::Rgb(Rgb { r: c.0, g: c.1, b: c.2, icc_profile: None })
}

fn draw_text(
    ops: &mut Vec<Op>,
    font_id: &FontId,
    x: f32,
    y: f32,
    text: &str,
    size: f32,
    color: (f32, f32, f32),
) {
    if text.is_empty() {
        return;
    }
    ops.push(Op::StartTextSection);
    ops.push(Op::SetFont { font: PdfFontHandle::External(font_id.clone()), size: Pt(size) });
    ops.push(Op::SetLineHeight { lh: Pt(size * 1.2) });
    ops.push(Op::SetFillColor { col: rgb(color) });
    ops.push(Op::SetTextCursor { pos: Point::new(Mm(x), Mm(y)) });
    ops.push(Op::ShowText { items: vec![TextItem::Text(text.to_string())] });
    ops.push(Op::EndTextSection);
}

/// Draws left-aligned text that wraps onto multiple lines within `max_width_mm`,
/// returning the y position immediately below the last line written.
fn draw_wrapped_text(
    ops: &mut Vec<Op>,
    font_id: &FontId,
    x: f32,
    y: f32,
    max_width_mm: f32,
    text: &str,
    size: f32,
    color: (f32, f32, f32),
) -> f32 {
    let line_height_mm = size * 1.35 * 0.3527;
    let lines = wrap_text(text, max_width_mm, size);
    let mut cy = y;
    for line in &lines {
        draw_text(ops, font_id, x, cy, line, size, color);
        cy -= line_height_mm;
    }
    cy
}

/// Very small greedy word-wrapper. DejaVu Sans at typical invoice sizes
/// averages roughly 0.52em per character, which is precise enough for
/// laying out addresses and free-text fields without pulling in full glyph
/// metrics.
fn wrap_text(text: &str, max_width_mm: f32, size_pt: f32) -> Vec<String> {
    let avg_char_width_mm = size_pt * 0.52 * 0.3527;
    let max_chars = ((max_width_mm / avg_char_width_mm).floor() as usize).max(8);

    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            let candidate_len = if current.is_empty() {
                word.len()
            } else {
                current.len() + 1 + word.len()
            };
            if candidate_len > max_chars && !current.is_empty() {
                lines.push(current.clone());
                current.clear();
            }
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn draw_hline(ops: &mut Vec<Op>, x1: f32, x2: f32, y: f32, thickness_pt: f32, color: (f32, f32, f32)) {
    ops.push(Op::SetOutlineColor { col: rgb(color) });
    ops.push(Op::SetOutlineThickness { pt: Pt(thickness_pt) });
    ops.push(Op::DrawLine {
        line: Line {
            points: vec![
                LinePoint { p: Point::new(Mm(x1), Mm(y)), bezier: false },
                LinePoint { p: Point::new(Mm(x2), Mm(y)), bezier: false },
            ],
            is_closed: false,
        },
    });
}

/// Draws a filled rectangle. Built from `Op::DrawPolygon` rather than
/// `Op::DrawRectangle`: printpdf 0.9.1's rectangle serializer
/// (`rectangle_to_stream_ops`) unconditionally ends the path with the PDF
/// `n` operator ("end path, no fill/stroke") and ignores `rectangle.mode`
/// entirely, so `Op::DrawRectangle` never actually paints anything. The
/// polygon serializer honours `mode` correctly.
fn draw_filled_rect(ops: &mut Vec<Op>, x: f32, y: f32, w: f32, h: f32, color: (f32, f32, f32)) {
    ops.push(Op::SetFillColor { col: rgb(color) });
    let corners = vec![
        LinePoint { p: Point::new(Mm(x), Mm(y)), bezier: false },
        LinePoint { p: Point::new(Mm(x + w), Mm(y)), bezier: false },
        LinePoint { p: Point::new(Mm(x + w), Mm(y + h)), bezier: false },
        LinePoint { p: Point::new(Mm(x), Mm(y + h)), bezier: false },
    ];
    ops.push(Op::DrawPolygon {
        polygon: Polygon {
            rings: vec![PolygonRing { points: corners }],
            mode: PaintMode::Fill,
            winding_order: WindingOrder::NonZero,
        },
    });
}

fn money(amount: f64, currency: &str) -> String {
    // Simple thousands-separated formatting; keeps the PDF independent from
    // any locale library while still reading naturally in French.
    let cents = (amount.abs() * 100.0).round() as i64;
    let whole = cents / 100;
    let frac = cents % 100;
    let mut whole_str = whole.to_string();
    let mut grouped = String::new();
    while whole_str.len() > 3 {
        let split_at = whole_str.len() - 3;
        grouped = format!(" {}{}", &whole_str[split_at..], grouped);
        whole_str.truncate(split_at);
    }
    grouped = format!("{}{}", whole_str, grouped);
    let sign = if amount < 0.0 { "-" } else { "" };
    format!("{}{},{:02} {}", sign, grouped, frac, currency)
}

// ---------------------------------------------------------------------
// Section renderers
// ---------------------------------------------------------------------

fn render_header(
    ops: &mut Vec<Op>,
    doc: &mut PdfDocument,
    fonts: &InvoiceFonts,
    data: &InvoicePdfData,
    cursor: &mut PdfCursor,
    palette: &Palette,
) {
    let top = cursor.y;
    let mut logo_bottom = top;

    if let Some(logo_path) = data.settings.logo_path.as_deref() {
        if let Ok(bytes) = std::fs::read(logo_path) {
            let mut warnings = Vec::new();
            if let Ok(image) = RawImage::decode_from_bytes(&bytes, &mut warnings) {
                let target_width_mm: f32 = 32.0;
                let dpi = if image.width > 0 {
                    (image.width as f32) * 25.4 / target_width_mm
                } else {
                    300.0
                };
                let height_mm = if image.width > 0 {
                    (image.height as f32) * target_width_mm / (image.width as f32)
                } else {
                    20.0
                };
                let image_id = doc.add_image(&image);
                let translate_y = top - height_mm;
                ops.push(Op::UseXobject {
                    id: image_id,
                    transform: XObjectTransform {
                        translate_x: Some(Mm(MARGIN).into()),
                        translate_y: Some(Mm(translate_y).into()),
                        rotate: None,
                        scale_x: None,
                        scale_y: None,
                        dpi: Some(dpi),
                    },
                });
                logo_bottom = translate_y;
            }
        }
    }

    // Bailleur information block, right-aligned column starting at the
    // horizontal midpoint of the page.
    let info_x = PAGE_WIDTH / 2.0 + 5.0;
    let mut y = top;
    draw_text(ops, &fonts.bold, info_x, y, &data.settings.full_name, 13.0, DARK);
    y -= 5.5;
    if let Some(company) = data.settings.company_name.as_deref().filter(|s| !s.is_empty()) {
        draw_text(ops, &fonts.regular, info_x, y, company, 9.5, GREY);
        y -= 4.6;
    }
    y = draw_wrapped_text(ops, &fonts.regular, info_x, y, CONTENT_RIGHT - info_x, &data.settings.address, 9.5, GREY);
    let contact_line = format!("{}, {}", data.settings.city, data.settings.country);
    draw_text(ops, &fonts.regular, info_x, y, &contact_line, 9.5, GREY);
    y -= 4.6;
    let phone_email = format!("{}  -  {}", data.settings.phone, data.settings.email);
    draw_text(ops, &fonts.regular, info_x, y, &phone_email, 9.5, GREY);
    y -= 4.6;
    if let Some(tax) = data.settings.tax_number.as_deref().filter(|s| !s.is_empty()) {
        draw_text(ops, &fonts.regular, info_x, y, &format!("N. fiscal : {}", tax), 9.0, GREY);
        y -= 4.4;
    }

    let bottom = logo_bottom.min(y).min(top - 26.0);
    cursor.y = bottom - 6.0;
    draw_hline(ops, MARGIN, CONTENT_RIGHT, cursor.y, palette.header_rule_weight, palette.brand);
    cursor.y -= 10.0;
}

fn render_title_and_meta(
    ops: &mut Vec<Op>,
    fonts: &InvoiceFonts,
    data: &InvoicePdfData,
    cursor: &mut PdfCursor,
    palette: &Palette,
) {
    draw_text(ops, &fonts.bold, MARGIN, cursor.y, "FACTURE DE LOYER", 19.0, palette.brand);

    let meta_x = PAGE_WIDTH / 2.0 + 15.0;
    let mut y = cursor.y + 1.5;
    draw_text(ops, &fonts.regular, meta_x, y, "Numero de facture", 8.5, GREY);
    draw_text(ops, &fonts.bold, meta_x + 32.0, y, &data.invoice.invoice_number, 10.0, DARK);
    y -= 5.5;
    draw_text(ops, &fonts.regular, meta_x, y, "Date d'emission", 8.5, GREY);
    draw_text(ops, &fonts.regular, meta_x + 32.0, y, &data.invoice.issue_date, 9.5, DARK);
    y -= 5.5;
    draw_text(ops, &fonts.regular, meta_x, y, "Date d'echeance", 8.5, GREY);
    draw_text(ops, &fonts.regular, meta_x + 32.0, y, &data.invoice.due_date, 9.5, DARK);

    cursor.y -= 14.0;
}

fn render_tenant_and_property(
    ops: &mut Vec<Op>,
    fonts: &InvoiceFonts,
    data: &InvoicePdfData,
    cursor: &mut PdfCursor,
) {
    let top = cursor.y;
    draw_text(ops, &fonts.bold, MARGIN, top, "FACTURE A", 9.5, GREY);
    let tenant_name = format!("{} {}", data.tenant.first_name, data.tenant.last_name);
    let mut y = top - 5.5;
    draw_text(ops, &fonts.bold, MARGIN, y, &tenant_name, 11.5, DARK);
    y -= 5.2;
    y = draw_wrapped_text(ops, &fonts.regular, MARGIN, y, 80.0, &data.tenant.address, 9.5, GREY);
    draw_text(ops, &fonts.regular, MARGIN, y, &data.tenant.phone, 9.5, GREY);
    y -= 4.6;
    if let Some(email) = data.tenant.email.as_deref().filter(|s| !s.is_empty()) {
        draw_text(ops, &fonts.regular, MARGIN, y, email, 9.5, GREY);
        y -= 4.6;
    }

    let right_x = PAGE_WIDTH / 2.0 + 15.0;
    let mut ry = top;
    draw_text(ops, &fonts.bold, right_x, ry, "BIEN LOUE", 9.5, GREY);
    ry -= 5.5;
    ry = draw_wrapped_text(ops, &fonts.regular, right_x, ry, CONTENT_RIGHT - right_x, &data.invoice.property_address, 9.5, DARK);
    let period = format!("Periode : {}", month_year_fr(data.invoice.billing_month, data.invoice.billing_year));
    draw_text(ops, &fonts.regular, right_x, ry, &period, 9.5, GREY);
    ry -= 4.6;
    if let Some(desc) = data.invoice.description.as_deref().filter(|s| !s.is_empty()) {
        draw_wrapped_text(ops, &fonts.regular, right_x, ry, CONTENT_RIGHT - right_x, desc, 9.0, GREY);
    }

    cursor.y = y.min(ry) - 8.0;
}

fn render_charges_table(
    ops: &mut Vec<Op>,
    fonts: &InvoiceFonts,
    data: &InvoicePdfData,
    cursor: &mut PdfCursor,
    palette: &Palette,
) {
    let currency = &data.settings.currency;
    let table_top = cursor.y;
    let row_h = 8.0;
    let header_h = 9.0;
    let col_amount_x = CONTENT_RIGHT - 5.0;

    draw_filled_rect(ops, MARGIN, table_top - header_h, CONTENT_RIGHT - MARGIN, header_h, palette.table_header_bg);
    draw_text(ops, &fonts.bold, MARGIN + 3.0, table_top - header_h + 2.8, "DESCRIPTION", 9.5, palette.table_header_fg);
    draw_text(ops, &fonts.bold, col_amount_x - 22.0, table_top - header_h + 2.8, "MONTANT", 9.5, palette.table_header_fg);
    if palette.table_header_border {
        draw_hline(ops, MARGIN, CONTENT_RIGHT, table_top - header_h, 0.5, GREY);
    }

    let mut y = table_top - header_h;

    let mut rows: Vec<(String, f64)> = vec![
        ("Loyer mensuel".to_string(), data.invoice.rent_amount),
        ("Charges d'eau".to_string(), data.invoice.water_charge),
        ("Charges d'electricite".to_string(), data.invoice.electricity_charge),
    ];
    if data.invoice.other_charges > 0.0 {
        rows.push(("Autres frais".to_string(), data.invoice.other_charges));
    }
    if data.invoice.discount > 0.0 {
        rows.push(("Remise accordee".to_string(), -data.invoice.discount));
    }

    for (i, (label, amount)) in rows.iter().enumerate() {
        y -= row_h;
        if i % 2 == 1 {
            if let Some(zebra) = palette.zebra {
                draw_filled_rect(ops, MARGIN, y, CONTENT_RIGHT - MARGIN, row_h, zebra);
            }
        }
        draw_text(ops, &fonts.regular, MARGIN + 3.0, y + 2.6, label, 9.5, DARK);
        let amount_str = money(*amount, currency);
        draw_text(ops, &fonts.regular, col_amount_x - (amount_str.len() as f32 * 1.7), y + 2.6, &amount_str, 9.5, DARK);
    }

    draw_hline(ops, MARGIN, CONTENT_RIGHT, y, 0.6, GREY);
    cursor.y = y - 6.0;
}

fn render_totals(
    ops: &mut Vec<Op>,
    fonts: &InvoiceFonts,
    data: &InvoicePdfData,
    cursor: &mut PdfCursor,
) {
    let currency = &data.settings.currency;
    let label_x = PAGE_WIDTH / 2.0 + 20.0;
    let value_x = CONTENT_RIGHT - 5.0;
    let mut y = cursor.y;

    let line = |ops: &mut Vec<Op>, y: f32, label: &str, amount: f64, bold: bool, color: (f32, f32, f32)| {
        let font = if bold { &fonts.bold } else { &fonts.regular };
        draw_text(ops, font, label_x, y, label, if bold { 11.0 } else { 9.5 }, color);
        let amount_str = money(amount, currency);
        let size = if bold { 11.0 } else { 9.5 };
        draw_text(ops, font, value_x - (amount_str.len() as f32 * (size * 0.19)), y, &amount_str, size, color);
    };

    line(ops, y, "Total", data.invoice.total_amount, true, DARK);
    y -= 6.5;
    line(ops, y, "Montant paye", data.invoice.amount_paid, false, GREY);
    y -= 6.0;
    draw_hline(ops, label_x, CONTENT_RIGHT, y + 2.5, 0.6, GREY);
    let balance_color = if data.invoice.balance_due > 0.0 { (0.72, 0.11, 0.11) } else { (0.02, 0.47, 0.34) };
    line(ops, y, "Reste a payer", data.invoice.balance_due, true, balance_color);
    y -= 8.0;

    cursor.y = y;
}

fn render_payment_info(
    ops: &mut Vec<Op>,
    fonts: &InvoiceFonts,
    data: &InvoicePdfData,
    cursor: &mut PdfCursor,
) {
    let mut y = cursor.y;
    let method = payment_method_fr(&data.invoice.payment_method);
    let status = status_fr(&data.invoice.status);
    draw_text(ops, &fonts.regular, MARGIN, y, &format!("Mode de paiement : {}", method), 9.5, DARK);
    draw_text(ops, &fonts.bold, PAGE_WIDTH / 2.0 + 20.0, y, &format!("Statut : {}", status), 9.5, DARK);
    y -= 6.5;

    if let Some(iban) = data.settings.iban.as_deref().filter(|s| !s.is_empty()) {
        draw_text(ops, &fonts.regular, MARGIN, y, &format!("IBAN : {}", iban), 9.0, GREY);
        y -= 5.5;
    }

    if let Some(obs) = data.invoice.observations.as_deref().filter(|s| !s.is_empty()) {
        draw_text(ops, &fonts.regular, MARGIN, y, "Observations :", 8.5, GREY);
        y -= 4.6;
        y = draw_wrapped_text(ops, &fonts.regular, MARGIN, y, CONTENT_RIGHT - MARGIN, obs, 9.0, GREY);
    }

    cursor.y = y;
}

fn render_signature(ops: &mut Vec<Op>, doc: &mut PdfDocument, fonts: &InvoiceFonts, data: &InvoicePdfData) {
    let block_x = CONTENT_RIGHT - 55.0;
    let mut y = 55.0;

    draw_text(ops, &fonts.regular, block_x, y, "Signature du bailleur", 9.0, GREY);
    y -= 4.0;

    if let Some(sig_path) = data.settings.signature_path.as_deref() {
        if let Ok(bytes) = std::fs::read(sig_path) {
            let mut warnings = Vec::new();
            if let Ok(image) = RawImage::decode_from_bytes(&bytes, &mut warnings) {
                let target_width_mm: f32 = 40.0;
                let dpi = if image.width > 0 {
                    (image.width as f32) * 25.4 / target_width_mm
                } else {
                    300.0
                };
                let height_mm = if image.width > 0 {
                    (image.height as f32) * target_width_mm / (image.width as f32)
                } else {
                    15.0
                };
                let image_id = doc.add_image(&image);
                ops.push(Op::UseXobject {
                    id: image_id,
                    transform: XObjectTransform {
                        translate_x: Some(Mm(block_x).into()),
                        translate_y: Some(Mm(y - height_mm).into()),
                        rotate: None,
                        scale_x: None,
                        scale_y: None,
                        dpi: Some(dpi),
                    },
                });
                y -= height_mm + 2.0;
            }
        }
    } else {
        y -= 14.0;
    }

    draw_hline(ops, block_x, CONTENT_RIGHT, y, 0.5, GREY);
    let today = data.invoice.issue_date.clone();
    draw_text(ops, &fonts.regular, block_x, y - 4.5, &format!("Fait le {}", today), 8.5, GREY);
}

fn render_footer(ops: &mut Vec<Op>, fonts: &InvoiceFonts, data: &InvoicePdfData) {
    draw_hline(ops, MARGIN, CONTENT_RIGHT, 20.0, 0.4, LIGHT_GREY);
    let footer_line = format!(
        "{}  -  {}  -  {}  -  {}",
        data.settings.full_name, data.settings.address, data.settings.phone, data.settings.email
    );
    draw_text(ops, &fonts.regular, MARGIN, 15.0, &footer_line, 7.5, GREY);
    draw_text(ops, &fonts.regular, MARGIN, 11.5, "Document genere localement - aucune donnee transmise en ligne.", 7.0, GREY);
}

fn month_year_fr(month: i64, year: i64) -> String {
    const NAMES: [&str; 12] = [
        "Janvier", "Fevrier", "Mars", "Avril", "Mai", "Juin",
        "Juillet", "Aout", "Septembre", "Octobre", "Novembre", "Decembre",
    ];
    let name = NAMES.get((month as usize).saturating_sub(1)).copied().unwrap_or("");
    format!("{} {}", name, year)
}

fn payment_method_fr(method: &str) -> &'static str {
    match method {
        "cash" => "Especes",
        "bank_transfer" => "Virement bancaire",
        "mobile_money" => "Mobile Money",
        "check" => "Cheque",
        _ => "Autre",
    }
}

fn status_fr(status: &str) -> &'static str {
    match status {
        "paid" => "Paye",
        "partially_paid" => "Partiellement paye",
        _ => "Non paye",
    }
}
