use printpdf::*;
use std::f32::consts::TAU;

use crate::models::{Invoice, Settings, Tenant};
use crate::pdf::fonts::InvoiceFonts;

/// A5 keeps the exact aspect ratio of A4 (ISO 216: 1:sqrt(2)), so every mm/pt
/// value in this file is the A4 design scaled down by this factor. This
/// preserves the layout proportions exactly instead of re-deriving them.
const SCALE: f32 = 0.7071;

const PAGE_WIDTH: f32 = 148.0;
const PAGE_HEIGHT: f32 = 210.0;
const MARGIN: f32 = 15.0 * SCALE;
const CONTENT_RIGHT: f32 = PAGE_WIDTH - MARGIN;

const DARK: (f32, f32, f32) = (0.117, 0.161, 0.231); // slate-900
const GREY: (f32, f32, f32) = (0.392, 0.455, 0.545); // slate-500
const LIGHT_GREY: (f32, f32, f32) = (0.945, 0.957, 0.976); // slate-100
const BORDER_GREY: (f32, f32, f32) = (0.851, 0.871, 0.902); // slate-200
const WHITE: (f32, f32, f32) = (1.0, 1.0, 1.0);
const GREEN: (f32, f32, f32) = (0.020, 0.470, 0.340);
const AMBER: (f32, f32, f32) = (0.710, 0.450, 0.050);
const RED: (f32, f32, f32) = (0.720, 0.110, 0.110);

/// Nominal half-size (in mm, already scaled) of the small pictograms drawn
/// next to labels throughout the invoice (contact rows, meta box, payment
/// box). Kept as a single constant so every icon reads at a consistent size.
const ICON_S: f32 = 1.7 * SCALE;

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
    /// Accent colour used for the title, header divider and pictograms.
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
    /// Colour used for the small pictograms (contact rows, meta box, payment
    /// box). Follows the brand colour except on Minimal, which stays
    /// monochrome.
    icon: (f32, f32, f32),
}

impl Palette {
    fn for_template(template: InvoiceTemplate) -> Self {
        match template {
            InvoiceTemplate::Classic => Palette {
                brand: (0.145, 0.388, 0.922), // #2563eb
                header_rule_weight: 1.2 * SCALE,
                table_header_bg: DARK,
                table_header_fg: WHITE,
                zebra: Some(LIGHT_GREY),
                table_header_border: false,
                icon: (0.145, 0.388, 0.922),
            },
            InvoiceTemplate::Modern => Palette {
                brand: (0.024, 0.467, 0.435), // #0c7768 teal
                header_rule_weight: 2.6 * SCALE,
                table_header_bg: (0.024, 0.467, 0.435),
                table_header_fg: WHITE,
                zebra: Some((0.902, 0.961, 0.949)), // teal-tinted slate-100
                table_header_border: false,
                icon: (0.024, 0.467, 0.435),
            },
            InvoiceTemplate::Minimal => Palette {
                brand: DARK, // monochrome: no accent colour
                header_rule_weight: 0.5 * SCALE,
                table_header_bg: WHITE,
                table_header_fg: DARK,
                zebra: None,
                table_header_border: true,
                icon: DARK,
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

/// Renders a complete, printable A5 rent invoice and returns the raw PDF bytes.
pub fn render_invoice_pdf(data: &InvoicePdfData) -> Result<Vec<u8>, String> {
    let mut doc = PdfDocument::new(&format!("Facture {}", data.invoice.invoice_number));
    let fonts = InvoiceFonts::load(&mut doc)?;
    let palette = Palette::for_template(InvoiceTemplate::from_key(&data.settings.invoice_template));

    let mut ops: Vec<Op> = Vec::new();
    let mut cursor = PdfCursor { y: PAGE_HEIGHT - MARGIN };

    render_header(&mut ops, &mut doc, &fonts, data, &mut cursor, &palette);
    render_title_and_meta(&mut ops, &fonts, data, &mut cursor, &palette);
    render_tenant(&mut ops, &fonts, data, &mut cursor, &palette);
    render_charges_table(&mut ops, &fonts, data, &mut cursor, &palette);
    render_totals(&mut ops, &fonts, data, &mut cursor, &palette);
    render_payment_info(&mut ops, &fonts, data, &mut cursor, &palette);
    render_signature(&mut ops, &mut doc, &fonts, data);
    render_footer(&mut ops, &fonts, data, &palette);

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

/// Lightens a colour towards white by `amount` (0 = unchanged, 1 = white).
/// Used to derive soft tinted backgrounds (e.g. the tenant avatar) from the
/// template's brand colour.
fn lighten(c: (f32, f32, f32), amount: f32) -> (f32, f32, f32) {
    (
        c.0 + (1.0 - c.0) * amount,
        c.1 + (1.0 - c.1) * amount,
        c.2 + (1.0 - c.2) * amount,
    )
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

/// Draws right-aligned text ending at `right_x`, using the same rough glyph
/// metrics as [`money`] callers already relied on.
fn draw_text_right(
    ops: &mut Vec<Op>,
    font_id: &FontId,
    right_x: f32,
    y: f32,
    text: &str,
    size: f32,
    color: (f32, f32, f32),
) {
    let width = text.len() as f32 * size * 0.24;
    draw_text(ops, font_id, right_x - width, y, text, size, color);
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
    draw_line(ops, x1, y, x2, y, thickness_pt, color);
}

/// Draws a horizontal dotted rule (used as the row separators inside the
/// bordered info boxes), restoring a solid dash pattern afterwards so it
/// doesn't leak into unrelated lines drawn later.
fn draw_dotted_hline(ops: &mut Vec<Op>, x1: f32, x2: f32, y: f32, thickness_pt: f32, color: (f32, f32, f32)) {
    ops.push(Op::SetLineDashPattern {
        dash: LineDashPattern { offset: 0, dash_1: Some(1), gap_1: Some(1), ..Default::default() },
    });
    draw_hline(ops, x1, x2, y, thickness_pt, color);
    ops.push(Op::SetLineDashPattern { dash: LineDashPattern::default() });
}

/// Draws a straight line between two arbitrary points.
fn draw_line(ops: &mut Vec<Op>, x1: f32, y1: f32, x2: f32, y2: f32, thickness_pt: f32, color: (f32, f32, f32)) {
    ops.push(Op::SetOutlineColor { col: rgb(color) });
    ops.push(Op::SetOutlineThickness { pt: Pt(thickness_pt) });
    ops.push(Op::DrawLine {
        line: Line {
            points: vec![
                LinePoint { p: Point::new(Mm(x1), Mm(y1)), bezier: false },
                LinePoint { p: Point::new(Mm(x2), Mm(y2)), bezier: false },
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
    ops.push(Op::DrawPolygon {
        polygon: Polygon {
            rings: vec![PolygonRing { points: rect_points(x, y, w, h) }],
            mode: PaintMode::Fill,
            winding_order: WindingOrder::NonZero,
        },
    });
}

/// Draws the outline of a rectangle (see [`draw_filled_rect`] for why this
/// goes through `Op::DrawPolygon` rather than `Op::DrawRectangle`).
fn draw_rect_stroke(ops: &mut Vec<Op>, x: f32, y: f32, w: f32, h: f32, thickness_pt: f32, color: (f32, f32, f32)) {
    ops.push(Op::SetOutlineColor { col: rgb(color) });
    ops.push(Op::SetOutlineThickness { pt: Pt(thickness_pt) });
    ops.push(Op::DrawPolygon {
        polygon: Polygon {
            rings: vec![PolygonRing { points: rect_points(x, y, w, h) }],
            mode: PaintMode::Stroke,
            winding_order: WindingOrder::NonZero,
        },
    });
}

fn rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<LinePoint> {
    vec![
        LinePoint { p: Point::new(Mm(x), Mm(y)), bezier: false },
        LinePoint { p: Point::new(Mm(x + w), Mm(y)), bezier: false },
        LinePoint { p: Point::new(Mm(x + w), Mm(y + h)), bezier: false },
        LinePoint { p: Point::new(Mm(x), Mm(y + h)), bezier: false },
    ]
}

fn draw_triangle_fill(ops: &mut Vec<Op>, p1: (f32, f32), p2: (f32, f32), p3: (f32, f32), color: (f32, f32, f32)) {
    ops.push(Op::SetFillColor { col: rgb(color) });
    let points = [p1, p2, p3]
        .iter()
        .map(|(x, y)| LinePoint { p: Point::new(Mm(*x), Mm(*y)), bezier: false })
        .collect();
    ops.push(Op::DrawPolygon {
        polygon: Polygon {
            rings: vec![PolygonRing { points }],
            mode: PaintMode::Fill,
            winding_order: WindingOrder::NonZero,
        },
    });
}

/// Approximates a circle as a 24-sided polygon; printpdf has no native
/// ellipse primitive.
fn circle_ring(cx: f32, cy: f32, r: f32) -> Vec<LinePoint> {
    const SEGMENTS: usize = 24;
    (0..SEGMENTS)
        .map(|i| {
            let angle = (i as f32 / SEGMENTS as f32) * TAU;
            LinePoint {
                p: Point::new(Mm(cx + r * angle.cos()), Mm(cy + r * angle.sin())),
                bezier: false,
            }
        })
        .collect()
}

fn draw_circle_fill(ops: &mut Vec<Op>, cx: f32, cy: f32, r: f32, color: (f32, f32, f32)) {
    if r <= 0.0 {
        return;
    }
    ops.push(Op::SetFillColor { col: rgb(color) });
    ops.push(Op::DrawPolygon {
        polygon: Polygon {
            rings: vec![PolygonRing { points: circle_ring(cx, cy, r) }],
            mode: PaintMode::Fill,
            winding_order: WindingOrder::NonZero,
        },
    });
}

fn draw_circle_stroke(ops: &mut Vec<Op>, cx: f32, cy: f32, r: f32, thickness_pt: f32, color: (f32, f32, f32)) {
    if r <= 0.0 {
        return;
    }
    ops.push(Op::SetOutlineColor { col: rgb(color) });
    ops.push(Op::SetOutlineThickness { pt: Pt(thickness_pt) });
    ops.push(Op::DrawPolygon {
        polygon: Polygon {
            rings: vec![PolygonRing { points: circle_ring(cx, cy, r) }],
            mode: PaintMode::Stroke,
            winding_order: WindingOrder::NonZero,
        },
    });
}

/// Clips subsequent drawing to a circle. Must be paired with
/// `Op::SaveGraphicsState` before and `Op::RestoreGraphicsState` after, or
/// the clip leaks into unrelated content.
fn clip_to_circle(ops: &mut Vec<Op>, cx: f32, cy: f32, r: f32) {
    ops.push(Op::DrawPolygon {
        polygon: Polygon {
            rings: vec![PolygonRing { points: circle_ring(cx, cy, r) }],
            mode: PaintMode::Clip,
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
// Pictograms
// ---------------------------------------------------------------------

/// The small line-icons used throughout the invoice to lead contact rows,
/// meta-box rows and the payment box. printpdf has no icon font or SVG
/// support, so each glyph is hand-built from circles, triangles, rectangles
/// and line segments sized around [`ICON_S`].
#[derive(Clone, Copy)]
enum Icon {
    Pin,
    Phone,
    Mail,
    Doc,
    Calendar,
    Home,
    Clock,
    Card,
}

fn draw_icon(ops: &mut Vec<Op>, icon: Icon, cx: f32, cy: f32, color: (f32, f32, f32)) {
    let s = ICON_S;
    match icon {
        Icon::Pin => {
            draw_triangle_fill(ops, (cx - s * 0.5, cy + s * 0.05), (cx + s * 0.5, cy + s * 0.05), (cx, cy - s * 0.9), color);
            draw_circle_fill(ops, cx, cy + s * 0.15, s * 0.62, color);
            draw_circle_fill(ops, cx, cy + s * 0.15, s * 0.24, WHITE);
        }
        Icon::Phone => {
            draw_circle_fill(ops, cx, cy, s * 0.55, color);
        }
        Icon::Mail => {
            draw_rect_stroke(ops, cx - s * 0.85, cy - s * 0.6, s * 1.7, s * 1.2, 0.45 * SCALE, color);
            draw_line(ops, cx - s * 0.78, cy + s * 0.52, cx, cy - s * 0.05, 0.45 * SCALE, color);
            draw_line(ops, cx + s * 0.78, cy + s * 0.52, cx, cy - s * 0.05, 0.45 * SCALE, color);
        }
        Icon::Doc => {
            draw_rect_stroke(ops, cx - s * 0.6, cy - s * 0.85, s * 1.2, s * 1.7, 0.45 * SCALE, color);
            draw_line(ops, cx - s * 0.32, cy - s * 0.05, cx + s * 0.32, cy - s * 0.05, 0.4 * SCALE, color);
            draw_line(ops, cx - s * 0.32, cy + s * 0.35, cx + s * 0.32, cy + s * 0.35, 0.4 * SCALE, color);
        }
        Icon::Calendar => {
            draw_rect_stroke(ops, cx - s * 0.85, cy - s * 0.75, s * 1.7, s * 1.5, 0.45 * SCALE, color);
            draw_line(ops, cx - s * 0.85, cy + s * 0.3, cx + s * 0.85, cy + s * 0.3, 0.45 * SCALE, color);
            draw_line(ops, cx - s * 0.4, cy + s * 0.75, cx - s * 0.4, cy + s * 0.95, 0.45 * SCALE, color);
            draw_line(ops, cx + s * 0.4, cy + s * 0.75, cx + s * 0.4, cy + s * 0.95, 0.45 * SCALE, color);
        }
        Icon::Home => {
            draw_triangle_fill(ops, (cx - s * 0.9, cy), (cx + s * 0.9, cy), (cx, cy + s * 0.85), color);
            draw_filled_rect(ops, cx - s * 0.6, cy - s * 0.9, s * 1.2, s * 0.9, color);
        }
        Icon::Clock => {
            draw_circle_stroke(ops, cx, cy, s * 0.9, 0.45 * SCALE, color);
            draw_line(ops, cx, cy, cx, cy + s * 0.5, 0.45 * SCALE, color);
            draw_line(ops, cx, cy, cx + s * 0.4, cy, 0.45 * SCALE, color);
        }
        Icon::Card => {
            draw_rect_stroke(ops, cx - s * 0.9, cy - s * 0.6, s * 1.8, s * 1.2, 0.45 * SCALE, color);
            draw_filled_rect(ops, cx - s * 0.9, cy + s * 0.1, s * 1.8, s * 0.3, color);
        }
    }
}

/// Draws a checkmark inside a circle outline, used for the payment status.
fn draw_icon_check(ops: &mut Vec<Op>, cx: f32, cy: f32, r: f32, color: (f32, f32, f32)) {
    draw_circle_stroke(ops, cx, cy, r, 0.6 * SCALE, color);
    draw_line(ops, cx - r * 0.42, cy - r * 0.02, cx - r * 0.08, cy - r * 0.4, 0.6 * SCALE, color);
    draw_line(ops, cx - r * 0.08, cy - r * 0.4, cx + r * 0.5, cy + r * 0.38, 0.6 * SCALE, color);
}

/// Draws a simple person silhouette (head + shoulders) clipped inside a
/// tinted circle, used as the tenant avatar placeholder.
fn draw_icon_avatar(ops: &mut Vec<Op>, cx: f32, cy: f32, r: f32, bg: (f32, f32, f32), fg: (f32, f32, f32)) {
    draw_circle_fill(ops, cx, cy, r, bg);
    ops.push(Op::SaveGraphicsState);
    clip_to_circle(ops, cx, cy, r);
    draw_circle_fill(ops, cx, cy - r * 0.5, r * 0.75, fg);
    ops.push(Op::RestoreGraphicsState);
    draw_circle_fill(ops, cx, cy + r * 0.28, r * 0.32, fg);
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
                let target_width_mm: f32 = 32.0 * SCALE;
                let dpi = if image.width > 0 {
                    (image.width as f32) * 25.4 / target_width_mm
                } else {
                    300.0
                };
                let height_mm = if image.width > 0 {
                    (image.height as f32) * target_width_mm / (image.width as f32)
                } else {
                    20.0 * SCALE
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
    let info_x = PAGE_WIDTH / 2.0 + 5.0 * SCALE;
    let mut y = top;
    draw_text(ops, &fonts.bold, info_x, y, &data.settings.full_name, 13.0 * SCALE, DARK);
    y -= 5.2 * SCALE;
    if let Some(company) = data.settings.company_name.as_deref().filter(|s| !s.is_empty()) {
        draw_text(ops, &fonts.regular, info_x, y, company, 9.5 * SCALE, GREY);
        y -= 4.2 * SCALE;
    }
    y = draw_wrapped_text(ops, &fonts.regular, info_x, y, CONTENT_RIGHT - info_x, &data.settings.address, 9.5 * SCALE, GREY);
    y -= 0.6 * SCALE;

    // Icon-led contact rows: location, phone, email and (optionally) tax number.
    let icon_cx = info_x + ICON_S * 1.1;
    let text_x = info_x + 4.6 * SCALE;
    let row_gap = 4.1 * SCALE;
    let contact_size = 9.2 * SCALE;

    let contact_row = |ops: &mut Vec<Op>, y: f32, icon: Icon, text: &str, size: f32| {
        let icon_cy = y + size * 0.3527 * 0.32;
        draw_icon(ops, icon, icon_cx, icon_cy, palette.icon);
        draw_text(ops, &fonts.regular, text_x, y, text, size, GREY);
    };

    let contact_line = format!("{}, {}", data.settings.city, data.settings.country);
    contact_row(ops, y, Icon::Pin, &contact_line, contact_size);
    y -= row_gap;
    contact_row(ops, y, Icon::Phone, &data.settings.phone, contact_size);
    y -= row_gap;
    contact_row(ops, y, Icon::Mail, &data.settings.email, contact_size);
    y -= row_gap;
    if let Some(tax) = data.settings.tax_number.as_deref().filter(|s| !s.is_empty()) {
        contact_row(ops, y, Icon::Doc, &format!("N. fiscal : {}", tax), 8.8 * SCALE);
        y -= row_gap;
    }

    let bottom = logo_bottom.min(y).min(top - 26.0 * SCALE);

    // Thin vertical rule separating the logo column from the bailleur block.
    let divider_x = MARGIN + 44.0 * SCALE;
    draw_line(ops, divider_x, top, divider_x, bottom, 0.4 * SCALE, BORDER_GREY);

    cursor.y = bottom - 5.0 * SCALE;
    draw_hline(ops, MARGIN, CONTENT_RIGHT, cursor.y, palette.header_rule_weight, palette.brand);
    cursor.y -= 9.0 * SCALE;
}

fn render_title_and_meta(
    ops: &mut Vec<Op>,
    fonts: &InvoiceFonts,
    data: &InvoicePdfData,
    cursor: &mut PdfCursor,
    palette: &Palette,
) {
    let top = cursor.y;
    draw_text(ops, &fonts.bold, MARGIN, top, "FACTURE DE LOYER", 19.0 * SCALE, palette.brand);
    let underline_y = top - 6.5 * SCALE;
    draw_hline(ops, MARGIN, MARGIN + 20.0 * SCALE, underline_y, 2.2 * SCALE, palette.brand);
    let title_bottom = underline_y;

    // Bordered meta box: invoice number, dates, property and billing period.
    let box_left = PAGE_WIDTH / 2.0 + 3.0 * SCALE;
    let box_right = CONTENT_RIGHT;
    let box_pad = 3.0 * SCALE;
    let icon_cx = box_left + box_pad + ICON_S * 1.1;
    let label_x = box_left + box_pad + 5.2 * SCALE;
    let value_x = box_left + box_pad + 28.0 * SCALE;
    let value_w = box_right - box_pad - value_x;
    let label_size = 7.2 * SCALE;
    let value_size = 9.0 * SCALE;

    let mut rows: Vec<(Icon, &str, String, bool)> = vec![
        (Icon::Doc, "N. FACTURE", data.invoice.invoice_number.clone(), true),
        (Icon::Calendar, "EMISSION", data.invoice.issue_date.clone(), false),
        (Icon::Calendar, "ECHEANCE", data.invoice.due_date.clone(), false),
        (Icon::Home, "BIEN LOUE", data.invoice.property_address.clone(), false),
        (Icon::Clock, "PERIODE", month_year_fr(data.invoice.billing_month, data.invoice.billing_year), false),
    ];
    if let Some(desc) = data.invoice.description.as_deref().filter(|s| !s.is_empty()) {
        rows.push((Icon::Doc, "DESCRIPTION", desc.to_string(), false));
    }

    let box_top = top + 2.5 * SCALE;
    let mut y = box_top - box_pad - 2.0 * SCALE;
    let row_count = rows.len();
    for (i, (icon, label, value, bold)) in rows.iter().enumerate() {
        let icon_cy = y + value_size * 0.3527 * 0.32;
        draw_icon(ops, *icon, icon_cx, icon_cy, palette.icon);
        draw_text(ops, &fonts.regular, label_x, y, label, label_size, GREY);
        let value_font = if *bold { &fonts.bold } else { &fonts.regular };
        let after = draw_wrapped_text(ops, value_font, value_x, y, value_w, value, value_size, DARK);
        if i + 1 < row_count {
            let sep_y = after + 1.6 * SCALE;
            draw_dotted_hline(ops, label_x, box_right - box_pad, sep_y, 0.4 * SCALE, BORDER_GREY);
            y = sep_y - 3.0 * SCALE;
        } else {
            y = after;
        }
    }
    let box_bottom = y - box_pad + 1.4 * SCALE;
    draw_rect_stroke(ops, box_left, box_bottom, box_right - box_left, box_top - box_bottom, 0.5 * SCALE, BORDER_GREY);

    cursor.y = title_bottom.min(box_bottom) - 8.0 * SCALE;
}

fn render_tenant(
    ops: &mut Vec<Op>,
    fonts: &InvoiceFonts,
    data: &InvoicePdfData,
    cursor: &mut PdfCursor,
    palette: &Palette,
) {
    let top = cursor.y;
    draw_text(ops, &fonts.bold, MARGIN, top, "FACTURE A", 9.5 * SCALE, GREY);

    let avatar_r = 5.6 * SCALE;
    let avatar_cy = top - 5.0 * SCALE - avatar_r * 0.55;
    let avatar_cx = MARGIN + avatar_r;
    draw_icon_avatar(ops, avatar_cx, avatar_cy, avatar_r, lighten(palette.brand, 0.85), palette.brand);

    let text_x = MARGIN + avatar_r * 2.0 + 4.0 * SCALE;
    let tenant_name = format!("{} {}", data.tenant.first_name, data.tenant.last_name);
    let mut y = top - 5.5 * SCALE;
    draw_text(ops, &fonts.bold, text_x, y, &tenant_name, 11.5 * SCALE, DARK);
    y -= 5.0 * SCALE;
    y = draw_wrapped_text(ops, &fonts.regular, text_x, y, CONTENT_RIGHT - text_x, &data.tenant.address, 9.5 * SCALE, GREY);
    draw_text(ops, &fonts.regular, text_x, y, &data.tenant.phone, 9.5 * SCALE, GREY);
    y -= 4.4 * SCALE;
    if let Some(email) = data.tenant.email.as_deref().filter(|s| !s.is_empty()) {
        draw_text(ops, &fonts.regular, text_x, y, email, 9.5 * SCALE, GREY);
        y -= 4.4 * SCALE;
    }

    let avatar_bottom = avatar_cy - avatar_r;
    cursor.y = y.min(avatar_bottom) - 7.0 * SCALE;
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
    let row_h = 7.6 * SCALE;
    let header_h = 8.6 * SCALE;
    let col_amount_x = CONTENT_RIGHT - 5.0 * SCALE;

    draw_filled_rect(ops, MARGIN, table_top - header_h, CONTENT_RIGHT - MARGIN, header_h, palette.table_header_bg);
    draw_text(ops, &fonts.bold, MARGIN + 3.0 * SCALE, table_top - header_h + 2.7 * SCALE, "DESCRIPTION", 9.5 * SCALE, palette.table_header_fg);
    let amount_header = format!("MONTANT ({})", currency);
    draw_text_right(ops, &fonts.bold, col_amount_x, table_top - header_h + 2.7 * SCALE, &amount_header, 9.5 * SCALE, palette.table_header_fg);
    if palette.table_header_border {
        draw_hline(ops, MARGIN, CONTENT_RIGHT, table_top - header_h, 0.5 * SCALE, GREY);
    }

    let mut y = table_top - header_h;

    let mut rows: Vec<(String, f64)> = vec![
        ("Loyer mensuel".to_string(), data.invoice.rent_amount),
    ];
    // Water/electricity no longer get their own printed lines, but still fold
    // into "Autres frais" (alongside any other charge) so the total always
    // reconciles with what's shown, even for older invoices that have them set.
    let other = data.invoice.other_charges + data.invoice.water_charge + data.invoice.electricity_charge;
    if other > 0.0 {
        rows.push(("Autres frais".to_string(), other));
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
        draw_text(ops, &fonts.regular, MARGIN + 3.0 * SCALE, y + 2.5 * SCALE, label, 9.5 * SCALE, DARK);
        let amount_str = money(*amount, currency);
        draw_text_right(ops, &fonts.regular, col_amount_x, y + 2.5 * SCALE, &amount_str, 9.5 * SCALE, DARK);
    }

    draw_hline(ops, MARGIN, CONTENT_RIGHT, y, 0.6 * SCALE, GREY);
    cursor.y = y - 5.5 * SCALE;
}

fn render_totals(
    ops: &mut Vec<Op>,
    fonts: &InvoiceFonts,
    data: &InvoicePdfData,
    cursor: &mut PdfCursor,
    palette: &Palette,
) {
    let currency = &data.settings.currency;
    let top = cursor.y;
    let bar_h = 10.5 * SCALE;
    let split_x = PAGE_WIDTH / 2.0 + 3.0 * SCALE;

    draw_filled_rect(ops, MARGIN, top - bar_h, split_x - MARGIN, bar_h, LIGHT_GREY);
    draw_filled_rect(ops, split_x, top - bar_h, CONTENT_RIGHT - split_x, bar_h, palette.brand);
    let text_y = top - bar_h + bar_h * 0.32;
    draw_text(ops, &fonts.bold, MARGIN + 4.0 * SCALE, text_y, "TOTAL", 12.0 * SCALE, DARK);
    let amount_str = money(data.invoice.total_amount, currency);
    draw_text_right(ops, &fonts.bold, CONTENT_RIGHT - 4.0 * SCALE, text_y, &amount_str, 12.0 * SCALE, WHITE);

    let mut y = top - bar_h - 5.5 * SCALE;
    draw_text(ops, &fonts.regular, split_x, y, "Montant paye", 9.0 * SCALE, GREY);
    let amount_paid_str = money(data.invoice.amount_paid, currency);
    draw_text_right(ops, &fonts.regular, CONTENT_RIGHT, y, &amount_paid_str, 9.0 * SCALE, DARK);
    y -= 7.5 * SCALE;

    cursor.y = y;
}

fn render_payment_info(
    ops: &mut Vec<Op>,
    fonts: &InvoiceFonts,
    data: &InvoicePdfData,
    cursor: &mut PdfCursor,
    palette: &Palette,
) {
    let top = cursor.y;
    let box_w = 68.0 * SCALE;
    let box_left = MARGIN;
    let box_right = MARGIN + box_w;
    let pad = 3.0 * SCALE;
    let icon_cx = box_left + pad + ICON_S * 1.1;
    let label_x = box_left + pad + 5.2 * SCALE;

    let mut y = top - pad - 1.5 * SCALE;
    let field_label_y = y;
    draw_icon(ops, Icon::Card, icon_cx, field_label_y + 9.0 * SCALE * 0.3527 * 0.32, palette.icon);
    draw_text(ops, &fonts.regular, label_x, y, "MODE DE PAIEMENT", 8.0 * SCALE, GREY);
    y -= 4.4 * SCALE;
    draw_text(ops, &fonts.bold, label_x, y, payment_method_fr(&data.invoice.payment_method), 9.5 * SCALE, DARK);
    y -= 3.6 * SCALE;

    let iban = data.settings.iban.as_deref().filter(|s| !s.is_empty());
    if iban.is_some() {
        draw_dotted_hline(ops, label_x, box_right - pad, y, 0.4 * SCALE, BORDER_GREY);
        y -= 3.6 * SCALE;
        draw_text(ops, &fonts.regular, label_x, y, "IBAN", 8.0 * SCALE, GREY);
        y -= 4.4 * SCALE;
        draw_text(ops, &fonts.bold, label_x, y, iban.unwrap(), 9.5 * SCALE, DARK);
        y -= 3.6 * SCALE;
    }

    let box_bottom = y - pad + 2.0 * SCALE;
    draw_rect_stroke(ops, box_left, box_bottom, box_w, top - box_bottom, 0.5 * SCALE, BORDER_GREY);

    // Payment status, to the right of the box.
    let (status_color, status_label) = status_style(&data.invoice.status);
    let status_cx = box_right + 16.0 * SCALE;
    let status_r = ICON_S * 1.5;
    let status_cy = top - pad - status_r;
    draw_icon_check(ops, status_cx, status_cy, status_r, status_color);
    let status_text_x = status_cx + status_r + 3.0 * SCALE;
    draw_text(ops, &fonts.regular, status_text_x, status_cy + 1.8 * SCALE, "STATUT", 8.5 * SCALE, GREY);
    draw_text(ops, &fonts.bold, status_text_x, status_cy - 3.6 * SCALE, status_label, 10.5 * SCALE, DARK);

    let mut bottom = box_bottom;

    if let Some(obs) = data.invoice.observations.as_deref().filter(|s| !s.is_empty()) {
        let obs_y = bottom - 5.5 * SCALE;
        draw_text(ops, &fonts.regular, MARGIN, obs_y, "Observations :", 8.5 * SCALE, GREY);
        let after = draw_wrapped_text(ops, &fonts.regular, MARGIN, obs_y - 4.2 * SCALE, CONTENT_RIGHT - MARGIN, obs, 9.0 * SCALE, GREY);
        bottom = after;
    }

    cursor.y = bottom;
}

fn render_signature(ops: &mut Vec<Op>, doc: &mut PdfDocument, fonts: &InvoiceFonts, data: &InvoicePdfData) {
    let block_x = CONTENT_RIGHT - 55.0 * SCALE;
    let mut y = 55.0 * SCALE;

    draw_text(ops, &fonts.regular, block_x, y, "Signature du bailleur", 9.0 * SCALE, GREY);
    y -= 4.0 * SCALE;

    if let Some(sig_path) = data.settings.signature_path.as_deref() {
        if let Ok(bytes) = std::fs::read(sig_path) {
            let mut warnings = Vec::new();
            if let Ok(image) = RawImage::decode_from_bytes(&bytes, &mut warnings) {
                let target_width_mm: f32 = 40.0 * SCALE;
                let dpi = if image.width > 0 {
                    (image.width as f32) * 25.4 / target_width_mm
                } else {
                    300.0
                };
                let height_mm = if image.width > 0 {
                    (image.height as f32) * target_width_mm / (image.width as f32)
                } else {
                    15.0 * SCALE
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
                y -= height_mm + 2.0 * SCALE;
            }
        }
    } else {
        y -= 14.0 * SCALE;
    }

    draw_hline(ops, block_x, CONTENT_RIGHT, y, 0.5 * SCALE, GREY);
}

fn render_footer(ops: &mut Vec<Op>, fonts: &InvoiceFonts, data: &InvoicePdfData, palette: &Palette) {
    draw_hline(ops, MARGIN, CONTENT_RIGHT, 20.0 * SCALE, 0.4 * SCALE, LIGHT_GREY);

    let size = 7.5 * SCALE;
    let made_on = format!("Fait le {}", data.invoice.issue_date);
    // Reserve space for the right-aligned date first, then wrap the left
    // line to whatever's left so a long address/email can never run into
    // it (silently dropping trailing words rather than overlapping).
    let made_on_w = made_on.len() as f32 * size * 0.24;
    let footer_max_w = CONTENT_RIGHT - MARGIN - made_on_w - 4.0 * SCALE;
    let footer_line = format!(
        "{}  -  {}  -  {}  -  {}",
        data.settings.full_name, data.settings.address, data.settings.phone, data.settings.email
    );
    let footer_first_line = wrap_text(&footer_line, footer_max_w, size).remove(0);

    draw_text(ops, &fonts.regular, MARGIN, 15.0 * SCALE, &footer_first_line, size, GREY);
    draw_text_right(ops, &fonts.regular, CONTENT_RIGHT, 15.0 * SCALE, &made_on, size, palette.brand);
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

/// Colour and French label for an invoice's payment status, used for the
/// checkmark badge in the payment section.
fn status_style(status: &str) -> ((f32, f32, f32), &'static str) {
    match status {
        "paid" => (GREEN, "Paye"),
        "partially_paid" => (AMBER, "Partiellement paye"),
        _ => (RED, "Non paye"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_settings(template: &str) -> Settings {
        Settings {
            id: 1,
            full_name: "KORA Clement".to_string(),
            company_name: Some("CREDO".to_string()),
            address: "Patte d'oie".to_string(),
            phone: "+226 72 69 50 05".to_string(),
            email: "credobf@gmail.com".to_string(),
            city: "Ouagadougou".to_string(),
            country: "Burkina Faso".to_string(),
            currency: "XOF".to_string(),
            logo_path: None,
            signature_path: None,
            tax_number: Some("IFU52654896".to_string()),
            iban: Some("IBAN4578523".to_string()),
            additional_info: None,
            invoice_prefix: "LOY".to_string(),
            next_invoice_number: 2,
            date_format: "YYYY-MM-DD".to_string(),
            language: "fr".to_string(),
            theme: "light".to_string(),
            invoice_template: template.to_string(),
            updated_at: "2026-07-17".to_string(),
        }
    }

    fn dummy_tenant() -> Tenant {
        Tenant {
            id: 1,
            first_name: "Clement".to_string(),
            last_name: "BAMBARA".to_string(),
            phone: "+226 73 63 94 28".to_string(),
            email: Some("clement.bambara@gmail.com".to_string()),
            address: "Patte d'oie, Ouagadougou".to_string(),
            id_number: None,
            profession: None,
            notes: None,
            created_at: "2026-01-01".to_string(),
            updated_at: "2026-01-01".to_string(),
            invoice_count: None,
        }
    }

    fn dummy_invoice() -> Invoice {
        Invoice {
            id: 1,
            invoice_number: "LOY-2026-000001".to_string(),
            tenant_id: 1,
            tenant_name: "Clement BAMBARA".to_string(),
            property_address: "Ouagadougou, Patte d'oie".to_string(),
            description: None,
            billing_month: 7,
            billing_year: 2026,
            issue_date: "2026-07-17".to_string(),
            due_date: "2026-07-31".to_string(),
            rent_amount: 50000.0,
            water_charge: 0.0,
            electricity_charge: 0.0,
            other_charges: 0.0,
            discount: 0.0,
            total_amount: 50000.0,
            amount_paid: 50000.0,
            balance_due: 0.0,
            payment_method: "cash".to_string(),
            status: "paid".to_string(),
            observations: None,
            created_at: "2026-07-17".to_string(),
            updated_at: "2026-07-17".to_string(),
        }
    }

    #[test]
    fn renders_every_template() {
        for template in ["classic", "modern", "minimal"] {
            let settings = dummy_settings(template);
            let tenant = dummy_tenant();
            let invoice = dummy_invoice();
            let data = InvoicePdfData { settings: &settings, tenant: &tenant, invoice: &invoice };
            let bytes = render_invoice_pdf(&data).expect("render");
            assert!(bytes.starts_with(b"%PDF"), "{template} template did not produce a valid PDF");
        }
    }

    /// Worst-case content: every optional field absent/present in the
    /// opposite combination from the happy-path preview, long free-text
    /// fields that force wrapping, a partial payment, and every charge line
    /// populated. Exercises the layout's vertical budget without asserting
    /// on exact positions (printpdf doesn't expose layout back for
    /// inspection) - this is a smoke test against panics/malformed output.
    #[test]
    fn renders_worst_case_content_without_panicking() {
        let mut settings = dummy_settings("classic");
        settings.company_name = None;
        settings.tax_number = None;
        settings.iban = None;
        settings.address = "Zone industrielle, Secteur 15, non loin de la station essence, Ouagadougou".to_string();

        let mut tenant = dummy_tenant();
        tenant.email = None;
        tenant.address = "Quartier Somgande, rue 12.34, porte 567, non loin du marche central, Ouagadougou".to_string();

        let mut invoice = dummy_invoice();
        invoice.property_address = "Immeuble Le Baobab, 3eme etage, appartement 12, Avenue Kwame Nkrumah, Ouagadougou".to_string();
        invoice.description = Some("Loyer juillet incluant charges de copropriete et frais de gardiennage mensuel".to_string());
        invoice.water_charge = 5000.0;
        invoice.electricity_charge = 12500.0;
        invoice.other_charges = 3000.0;
        invoice.discount = 2000.0;
        invoice.total_amount = 68500.0;
        invoice.amount_paid = 30000.0;
        invoice.balance_due = 38500.0;
        invoice.payment_method = "bank_transfer".to_string();
        invoice.status = "partially_paid".to_string();
        invoice.observations = Some("Merci de regler le solde avant la fin du mois. Contactez le bailleur pour tout arrangement.".to_string());

        let data = InvoicePdfData { settings: &settings, tenant: &tenant, invoice: &invoice };
        let bytes = render_invoice_pdf(&data).expect("render");
        assert!(bytes.starts_with(b"%PDF"));
    }
}
