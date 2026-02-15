use web_sys::CanvasRenderingContext2d;

use crate::ui::orb::{hex_to_rgb, OrbData};

pub fn draw_terrain(
    ctx: &CanvasRenderingContext2d,
    width: f64,
    height: f64,
    orbs: &[OrbData],
    time: f64,
    dragging: Option<usize>,
) {
    ctx.set_fill_style_str("#0a0a0a");
    ctx.fill_rect(0.0, 0.0, width, height);

    // Breathing background
    let breath = 0.015 + 0.008 * (time * 0.4).sin();
    ctx.set_fill_style_str(&format!("rgba(0, 206, 209, {})", breath));
    ctx.fill_rect(0.0, 0.0, width, height);

    let warm_breath = 0.008 + 0.005 * (time * 0.25 + 1.5).sin();
    ctx.set_fill_style_str(&format!("rgba(147, 112, 219, {})", warm_breath));
    ctx.fill_rect(0.0, 0.0, width, height);

    let cx = width / 2.0;
    let cy = height / 2.0;

    // Concentric rings
    let max_r = (width.max(height)) * 0.55;
    let ring_count = 10;
    for i in 1..=ring_count {
        let r = max_r * (i as f64 / ring_count as f64);
        let alpha = 0.025 + 0.015 * ((time * 0.25 + i as f64 * 0.4).sin());
        ctx.begin_path();
        let _ = ctx.arc(cx, cy, r, 0.0, std::f64::consts::TAU);
        ctx.set_stroke_style_str(&format!("rgba(0, 206, 209, {})", alpha));
        ctx.set_line_width(0.5);
        ctx.stroke();
    }

    // Sonar pulse
    let pulse_cycle = (time * 0.3) % 1.0;
    let pulse_r = max_r * pulse_cycle;
    let pulse_alpha = 0.08 * (1.0 - pulse_cycle);
    ctx.begin_path();
    let _ = ctx.arc(cx, cy, pulse_r, 0.0, std::f64::consts::TAU);
    ctx.set_stroke_style_str(&format!("rgba(0, 206, 209, {})", pulse_alpha));
    ctx.set_line_width(1.5);
    ctx.stroke();

    // Axis labels
    ctx.set_font("10px monospace");
    ctx.set_fill_style_str("rgba(255,255,255,0.1)");
    let _ = ctx.fill_text("low", 12.0, height - 12.0);
    let _ = ctx.fill_text("high", width - 38.0, height - 12.0);
    ctx.save();
    let _ = ctx.translate(11.0, height / 2.0);
    let _ = ctx.rotate(-std::f64::consts::FRAC_PI_2);
    let _ = ctx.fill_text("amp", 0.0, 0.0);
    ctx.restore();

    // Connection lines
    if orbs.len() > 1 {
        for i in 0..orbs.len() {
            for j in (i + 1)..orbs.len() {
                let dx = orbs[j].x - orbs[i].x;
                let dy = orbs[j].y - orbs[i].y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < 300.0 {
                    let alpha = 0.08 * (1.0 - dist / 300.0);
                    let wave = 1.0 + 0.3 * (time * 1.5 + dist * 0.01).sin();
                    ctx.begin_path();
                    ctx.move_to(orbs[i].x, orbs[i].y);
                    ctx.line_to(orbs[j].x, orbs[j].y);
                    ctx.set_stroke_style_str(&format!(
                        "rgba(0, 206, 209, {})",
                        alpha * wave
                    ));
                    ctx.set_line_width(0.8);
                    ctx.stroke();
                }
            }
        }
    }

    for orb in orbs {
        draw_orb(ctx, orb, time, dragging == Some(orb.id));
    }
}

fn draw_orb(ctx: &CanvasRenderingContext2d, orb: &OrbData, time: f64, is_dragging: bool) {
    let pulse_speed = if orb.kind.is_noise() {
        1.2
    } else {
        (orb.freq.max(0.5) as f64 * 0.3).min(8.0)
    };
    let pulse = 1.0 + 0.06 * (time * pulse_speed + orb.pulse_phase).sin();
    let r = orb.radius * pulse;

    let (cr, cg, cb) = orb.rgb;

    // Outer glow
    let layers = 8;
    for i in (0..layers).rev() {
        let frac = (i as f64 + 1.0) / layers as f64;
        let layer_r = r * 0.3 + (r * 2.2 * frac);
        let alpha = 0.10 * (1.0 - frac) * (1.0 - frac);
        ctx.begin_path();
        let _ = ctx.arc(orb.x, orb.y, layer_r, 0.0, std::f64::consts::TAU);
        ctx.set_fill_style_str(&format!("rgba({},{},{},{})", cr, cg, cb, alpha));
        ctx.fill();
    }

    // Core
    let core_r = r * 0.22;
    ctx.begin_path();
    let _ = ctx.arc(orb.x, orb.y, core_r, 0.0, std::f64::consts::TAU);
    ctx.set_fill_style_str(&format!("rgba({},{},{},0.85)", cr, cg, cb));
    ctx.fill();

    // Specular highlight
    ctx.begin_path();
    let _ = ctx.arc(
        orb.x - core_r * 0.25,
        orb.y - core_r * 0.25,
        core_r * 0.35,
        0.0,
        std::f64::consts::TAU,
    );
    ctx.set_fill_style_str("rgba(255,255,255,0.35)");
    ctx.fill();

    // Filter rings (one per active filter)
    for (i, filter_kind) in orb.active_filters.iter().enumerate() {
        let (fr, fg, fb) = hex_to_rgb(filter_kind.color());
        let filter_pulse = 1.0 + 0.04 * (time * 1.8 + i as f64 * 0.5).sin();
        let filter_r = core_r * (3.2 + i as f64 * 0.9) * filter_pulse;

        ctx.begin_path();
        let _ = ctx.arc(orb.x, orb.y, filter_r, 0.0, std::f64::consts::TAU);
        ctx.set_stroke_style_str(&format!("rgba({},{},{},0.45)", fr, fg, fb));
        ctx.set_line_width(1.8);
        ctx.stroke();

        let outer_r = filter_r + 5.0;
        ctx.begin_path();
        let _ = ctx.arc(orb.x, orb.y, outer_r, 0.0, std::f64::consts::TAU);
        ctx.set_stroke_style_str(&format!("rgba({},{},{},0.15)", fr, fg, fb));
        ctx.set_line_width(0.8);
        ctx.stroke();

        ctx.begin_path();
        let _ = ctx.arc(orb.x, orb.y, filter_r, 0.0, std::f64::consts::TAU);
        ctx.set_fill_style_str(&format!("rgba({},{},{},0.03)", fr, fg, fb));
        ctx.fill();
    }

    // Binaural indicator
    if orb.binaural_active {
        let b_pulse = 1.0 + 0.12 * (time * 4.0).sin();
        let b_r = core_r * 2.5 * b_pulse;
        ctx.begin_path();
        let _ = ctx.arc(orb.x, orb.y, b_r, 0.0, std::f64::consts::TAU);
        ctx.set_stroke_style_str(&format!("rgba({},{},{},0.5)", cr, cg, cb));
        ctx.set_line_width(1.2);
        ctx.stroke();

        let b_pulse2 = 1.0 + 0.12 * (time * 4.0 + 1.0).sin();
        let b_r2 = core_r * 3.0 * b_pulse2;
        ctx.begin_path();
        let _ = ctx.arc(orb.x, orb.y, b_r2, 0.0, std::f64::consts::TAU);
        ctx.set_stroke_style_str(&format!("rgba({},{},{},0.25)", cr, cg, cb));
        ctx.set_line_width(0.8);
        ctx.stroke();
    }

    // Drag highlight + crosshairs
    if is_dragging {
        ctx.begin_path();
        let _ = ctx.arc(orb.x, orb.y, core_r * 2.2, 0.0, std::f64::consts::TAU);
        ctx.set_stroke_style_str("rgba(255,255,255,0.7)");
        ctx.set_line_width(1.5);
        ctx.stroke();

        ctx.set_stroke_style_str("rgba(255,255,255,0.06)");
        ctx.set_line_width(0.5);
        ctx.begin_path();
        ctx.move_to(0.0, orb.y);
        ctx.line_to(10000.0, orb.y);
        ctx.stroke();
        ctx.begin_path();
        ctx.move_to(orb.x, 0.0);
        ctx.line_to(orb.x, 10000.0);
        ctx.stroke();
    }
}
