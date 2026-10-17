/// Top-left of one preview diamond inside the preview box.
/// Grid +x runs down-right. Grid +y runs down-left.
pub fn diamond_offset(x: i32, y: i32, height: i32, tile_w: f32, tile_h: f32) -> (f32, f32) {
    let left = (x - y + height - 1) as f32 * (tile_w * 0.5);
    let top = (x + y) as f32 * (tile_h * 0.5);
    (left, top)
}

/// Width and height of the box that holds every diamond.
pub fn diamond_span(width: i32, height: i32, tile_w: f32, tile_h: f32) -> (f32, f32) {
    let span = (width + height) as f32;
    (span * tile_w * 0.5, span * tile_h * 0.5)
}

/// Even tile width, half that in height, fitted to the preview area.
pub fn preview_tile_size(width: i32, height: i32, avail_w: f32, avail_h: f32) -> (f32, f32) {
    let span = (width + height).max(1) as f32;
    let mut tile_w = (avail_w / span * 2.0)
        .min(avail_h / span * 4.0)
        .clamp(8.0, 64.0)
        .floor();
    if (tile_w as u32) % 2 == 1 {
        tile_w -= 1.0;
    }
    tile_w = tile_w.max(8.0);
    (tile_w, tile_w * 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_diamond_sits_inside_the_preview_box() {
        for (width, height) in [(1, 1), (6, 6), (12, 8)] {
            let (tile_w, tile_h) = preview_tile_size(width, height, 400.0, 760.0);
            let (span_w, span_h) = diamond_span(width, height, tile_w, tile_h);
            for y in 0..height {
                for x in 0..width {
                    let (left, top) = diamond_offset(x, y, height, tile_w, tile_h);
                    assert!(left >= -0.01, "{x},{y} left {left}");
                    assert!(top >= -0.01, "{x},{y} top {top}");
                    assert!(left + tile_w <= span_w + 0.01);
                    assert!(top + tile_h <= span_h + 0.01);
                }
            }
        }
    }
}
