// SPDX-License-Identifier: GPL-3.0-or-later
//! The Ulak mark (`assets/brand/icon.svg`): a U whose two arms, the phone
//! and the PC, meet at the bottom, with the message they carry as a spark
//! between them, on a night-blue tile with a soft glow. Drawn in code at any
//! pixel size: the tray, window and toast icons at runtime, and the
//! executable's icon at build time (`build.rs` includes this file), so every
//! size is sharp without bitmap assets. At 32 px and below the strokes are
//! heavier and the tile fills more of the square.

/// Night-blue tile, its glow, the sky rim, the two arms and the spark.
const TILE: [f32; 3] = rgb(0x10, 0x2A, 0x43);
const GLOW: [f32; 3] = rgb(0x1E, 0x4E, 0x79);
const RIM: [f32; 3] = rgb(0x5F, 0xB3, 0xF9);
const PHONE: [f32; 3] = rgb(0xF5, 0xF9, 0xFF);
const PC: [f32; 3] = rgb(0xCF, 0xE3, 0xFA);
const SPARK: [f32; 3] = rgb(0xFF, 0xB5, 0x47);

/// The U in a 100×100 design grid: straight arms from `ARM_TOP` down to
/// `BEND_Y`, then a half circle of radius `BEND_R` around (50, BEND_Y).
const ARM_X: f32 = 32.0;
const ARM_TOP: f32 = 24.0;
const BEND_Y: f32 = 56.0;
const BEND_R: f32 = 50.0 - ARM_X;
/// The spark between the arms' tops.
const SPARK_Y: f32 = 31.0;

/// Supersampling grid per pixel (SS×SS samples) for smooth edges.
const SS: usize = 4;

const fn rgb(r: u8, g: u8, b: u8) -> [f32; 3] {
    [r as f32, g as f32, b as f32]
}

/// Sizes in the design grid: the small sizes are drawn bolder.
struct Geometry {
    inset: f32,
    corner: f32,
    stroke: f32,
    spark: f32,
}

impl Geometry {
    fn for_size(size: usize) -> Self {
        if size <= 32 {
            Geometry { inset: 2.0, corner: 20.0, stroke: 13.0, spark: 8.0 }
        } else {
            Geometry { inset: 4.0, corner: 22.0, stroke: 11.0, spark: 7.2 }
        }
    }
}

/// Renders the mark as straight-alpha RGBA, row by row, `size`×`size`.
pub fn render(size: usize) -> Vec<u8> {
    let g = Geometry::for_size(size);
    let unit = 100.0 / size as f32;
    let left = arm(true);
    let right = arm(false);
    // The rim is at least about a pixel wide, so the tile keeps an edge on dark taskbars.
    let rim = (1.5f32).max(unit * 0.6);

    let mut out = vec![0u8; size * size * 4];
    for py in 0..size {
        for px in 0..size {
            let mut color = [0f32; 3];
            let mut covered = 0usize;
            for sy in 0..SS {
                for sx in 0..SS {
                    // The sample in the design grid.
                    let x = (px as f32 + (sx as f32 + 0.5) / SS as f32) * unit;
                    let y = (py as f32 + (sy as f32 + 0.5) / SS as f32) * unit;
                    let edge = rounded_square_distance(x, y, g.inset, g.corner);
                    if edge > 0.0 {
                        continue;
                    }
                    covered += 1;
                    let (dx, dy) = (x - 50.0, y - 50.0);
                    let half = g.stroke / 2.0;
                    let paint = if ((x - 50.0).powi(2) + (y - SPARK_Y).powi(2)).sqrt() <= g.spark {
                        SPARK
                    } else if distance_to(&left, (x, y)).min(distance_to(&right, (x, y))) <= half {
                        // The arms meet at the bottom in a straight seam.
                        if x < 50.0 { PHONE } else { PC }
                    } else if -edge <= rim {
                        mix(TILE, RIM, 0.34)
                    } else {
                        // The glow fades from the middle to the tile color at 40 units.
                        let t = ((dx * dx + dy * dy).sqrt() / 40.0).min(1.0);
                        mix(GLOW, TILE, t)
                    };
                    for c in 0..3 {
                        color[c] += paint[c];
                    }
                }
            }
            if covered == 0 {
                continue;
            }
            let i = (py * size + px) * 4;
            for c in 0..3 {
                out[i + c] = (color[c] / covered as f32).round() as u8;
            }
            out[i + 3] = ((covered as f32 / (SS * SS) as f32) * 255.0).round() as u8;
        }
    }
    out
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

/// How far (x, y) is outside the rounded square from `inset` to
/// `100 - inset` with corner radius `r` (negative inside).
fn rounded_square_distance(x: f32, y: f32, inset: f32, r: f32) -> f32 {
    let (lo, hi) = (inset + r, 100.0 - inset - r);
    let qx = (x.clamp(lo, hi) - x).abs();
    let qy = (y.clamp(lo, hi) - y).abs();
    if x > lo && x < hi || y > lo && y < hi {
        // Along a straight side.
        let to_x = (x - inset).min(100.0 - inset - x);
        let to_y = (y - inset).min(100.0 - inset - y);
        return -to_x.min(to_y);
    }
    (qx * qx + qy * qy).sqrt() - r
}

/// One arm of the U as points close enough together to measure against:
/// the left (phone) or the right (PC), each down to the bottom of the bend.
fn arm(left: bool) -> Vec<(f32, f32)> {
    const STEPS: usize = 32;
    let x = if left { ARM_X } else { 100.0 - ARM_X };
    let mut points = vec![(x, ARM_TOP), (x, BEND_Y)];
    for step in 1..=STEPS {
        // A quarter circle from the arm's end to the bottom of the U.
        let a = std::f32::consts::FRAC_PI_2 * step as f32 / STEPS as f32;
        let sx = if left { -a.cos() } else { a.cos() };
        points.push((50.0 + BEND_R * sx, BEND_Y + BEND_R * a.sin()));
    }
    points
}

fn distance_to(polyline: &[(f32, f32)], p: (f32, f32)) -> f32 {
    polyline.windows(2).map(|w| dist_to_segment(p, w[0], w[1])).fold(f32::INFINITY, f32::min)
}

fn dist_to_segment(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    let t = if len2 == 0.0 { 0.0 } else { (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2).clamp(0.0, 1.0) };
    let (qx, qy) = (a.0 + t * dx, a.1 + t * dy);
    ((p.0 - qx).powi(2) + (p.1 - qy).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(img: &[u8], size: usize, x: usize, y: usize) -> [u8; 4] {
        let i = (y * size + x) * 4;
        [img[i], img[i + 1], img[i + 2], img[i + 3]]
    }

    #[test]
    fn corners_are_transparent_and_the_spark_sits_between_the_arms() {
        let size = 100;
        let img = render(size);
        assert_eq!(img.len(), size * size * 4);
        assert_eq!(pixel(&img, size, 0, 0)[3], 0, "rounded corner");
        assert_eq!(pixel(&img, size, 50, 31), [0xFF, 0xB5, 0x47, 255], "the spark");
        // Near the top edge, above the spark: the dark tile.
        let tile = pixel(&img, size, 50, 8);
        assert!(tile[2] < 0x80 && tile[3] == 255, "{tile:?}");
        // Inside the U, below the spark: no ink.
        let inside = pixel(&img, size, 50, 52);
        assert!(inside[0] < 0x80, "{inside:?}");
    }

    #[test]
    fn both_arms_and_the_bend_are_drawn() {
        let size = 100;
        let img = render(size);
        let light = |p: [u8; 4]| p[0] > 0xC0 && p[1] > 0xD0 && p[3] == 255;
        assert!(light(pixel(&img, size, 32, 40)), "phone arm");
        assert!(light(pixel(&img, size, 68, 40)), "PC arm");
        assert!(light(pixel(&img, size, 50, 74)), "the bottom of the U");
    }

    #[test]
    fn renders_every_tray_size() {
        for size in [16, 20, 24, 32, 48, 64, 256] {
            let img = render(size);
            assert!(img.chunks(4).any(|p| p[3] == 255), "{size}px has an opaque tile");
        }
    }
}
