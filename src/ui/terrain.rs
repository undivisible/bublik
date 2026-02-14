use web_sys::CanvasRenderingContext2d;

use crate::ui::orb::OrbData;

pub fn draw_terrain(
    ctx: &CanvasRenderingContext2d,
    width: f64,
    height: f64,
    orbs: &[OrbData],
    time: f64,
    dragging: Option<usize>,
) {
    // Clear
    ctx.set_fill_style_str("#0a0a0a");
    ctx.fill_rect(0.0, 0.0, width, height);

    // Breathing background
    let breath_alpha = 0.02 + 0.01 * (time * 0.5).sin();
    ctx.set_fill_style_str(&format!("rgba(0, 206, 209, {})", breath_alpha));
    ctx.fill_rect(0.0, 0.0, width, height);

    // Concentric rings (radar/sonar)
    let cx = width / 2.0;
    let cy = height / 2.0;
    let max_r = (width.max(height)) * 0.6;
    let ring_count = 8;
    for i in 1..=ring_count {
        let r = max_r * (i as f64 / ring_count as f64);
        let alpha = 0.04 + 0.02 * ((time * 0.3 + i as f64 * 0.5).sin());
        ctx.begin_path();
        let _ = ctx.arc(cx, cy, r, 0.0, std::f64::consts::TAU);
        ctx.set_stroke_style_str(&format!("rgba(0, 206, 209, {})", alpha));
        ctx.set_line_width(1.0);
        ctx.stroke();
    }

    // Frequency axis labels
    ctx.set_font("11px monospace");
    ctx.set_fill_style_str("rgba(255,255,255,0.15)");
    let _ = ctx.fill_text("low freq", 10.0, height - 10.0);
    let _ = ctx.fill_text("high freq", width - 70.0, height - 10.0);
    ctx.save();
    let _ = ctx.translate(12.0, height / 2.0);
    let _ = ctx.rotate(-std::f64::consts::FRAC_PI_2);
    let _ = ctx.fill_text("amplitude", 0.0, 0.0);
    ctx.restore();

    // Draw connection lines between orbs
    if orbs.len() > 1 {
        for i in 0..orbs.len() {
            for j in (i + 1)..orbs.len() {
                let dx = orbs[j].x - orbs[i].x;
                let dy = orbs[j].y - orbs[i].y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < 250.0 {
                    let alpha = 0.1 * (1.0 - dist / 250.0);
                    ctx.begin_path();
                    ctx.move_to(orbs[i].x, orbs[i].y);
                    ctx.line_to(orbs[j].x, orbs[j].y);
                    ctx.set_stroke_style_str(&format!("rgba(0, 206, 209, {})", alpha));
                    ctx.set_line_width(1.0);
                    ctx.stroke();
                }
            }
        }
    }

    // Draw orbs
    for orb in orbs {
        draw_orb(ctx, orb, time, dragging == Some(orb.id));
    }
}

fn draw_orb(ctx: &CanvasRenderingContext2d, orb: &OrbData, time: f64, is_dragging: bool) {
    let pulse = 1.0 + 0.08 * (time * orb.freq.max(0.5) as f64 * 0.5 + orb.pulse_phase).sin();
    let r = orb.radius * pulse;

    let (cr, cg, cb) = hex_to_rgb(&orb.color);

    // Outer glow (layered circles for soft glow effect)
    let layers = 6;
    for i in (0..layers).rev() {
        let frac = (i as f64 + 1.0) / layers as f64;
        let layer_r = r + (r * 1.5 * frac);
        let alpha = 0.12 * (1.0 - frac);
        ctx.begin_path();
        let _ = ctx.arc(orb.x, orb.y, layer_r, 0.0, std::f64::consts::TAU);
        ctx.set_fill_style_str(&format!("rgba({},{},{},{})", cr, cg, cb, alpha));
        ctx.fill();
    }

    // Core orb
    ctx.begin_path();
    let _ = ctx.arc(orb.x, orb.y, r, 0.0, std::f64::consts::TAU);
    ctx.set_fill_style_str(&format!("rgba({},{},{},0.85)", cr, cg, cb));
    ctx.fill();

    // Inner highlight
    ctx.begin_path();
    let _ = ctx.arc(
        orb.x - r * 0.2,
        orb.y - r * 0.2,
        r * 0.4,
        0.0,
        std::f64::consts::TAU,
    );
    ctx.set_fill_style_str("rgba(255,255,255,0.15)");
    ctx.fill();

    // Drag highlight
    if is_dragging {
        ctx.begin_path();
        let _ = ctx.arc(orb.x, orb.y, r + 4.0, 0.0, std::f64::consts::TAU);
        ctx.set_stroke_style_str("rgba(255,255,255,0.5)");
        ctx.set_line_width(2.0);
        ctx.stroke();
    }

    // Label
    ctx.set_font("10px monospace");
    ctx.set_fill_style_str("rgba(255,255,255,0.7)");
    ctx.set_text_align("center");
    let _ = ctx.fill_text(orb.kind.label(), orb.x, orb.y + r + 16.0);
    ctx.set_text_align("start");
}

fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    if hex.len() < 7 {
        return (128, 128, 128);
    }
    let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(128);
    let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(128);
    let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(128);
    (r, g, b)
}
