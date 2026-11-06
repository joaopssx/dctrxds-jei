// Derived from Just Enough Items (https://github.com/mezz/JustEnoughItems),
// Common/src/main/java/mezz/jei/common/gui/GridScrollMath.java and its test.
// Copyright (c) 2014-2015 mezz. MIT licence: LICENSES/JustEnoughItems-MIT.txt.

//! Which items a scrolled grid shows. Offsets go from 0 (top) to 1 (bottom).

pub fn total_rows(item_count: i32, columns: i32) -> i32 {
    if item_count == 0 || columns == 0 {
        return 0;
    }
    (item_count + columns - 1) / columns
}

pub fn max_first_item_index(
    item_count: i32,
    columns: i32,
    visible_rows: i32,
    visible_item_count: i32,
) -> i32 {
    if item_count == 0 || columns == 0 || visible_rows == 0 || visible_item_count == 0 {
        return 0;
    }
    let rows = total_rows(item_count, columns);
    let row_based = (rows - visible_rows).max(0) * columns;
    let slot_based = (item_count - visible_item_count).max(0);
    row_based.max(slot_based).min(item_count - 1)
}

pub fn hidden_rows(
    item_count: i32,
    columns: i32,
    visible_rows: i32,
    visible_item_count: i32,
) -> i32 {
    total_rows(
        max_first_item_index(item_count, columns, visible_rows, visible_item_count),
        columns,
    )
}

pub fn valid_scroll_offset(
    offset: f32,
    item_count: i32,
    columns: i32,
    visible_rows: i32,
    visible_item_count: i32,
) -> f32 {
    if hidden_rows(item_count, columns, visible_rows, visible_item_count) == 0 {
        return 0.0;
    }
    offset.clamp(0.0, 1.0)
}

pub fn offset_for_first_row(
    first_row: i32,
    item_count: i32,
    columns: i32,
    visible_rows: i32,
    visible_item_count: i32,
) -> f32 {
    let hidden = hidden_rows(item_count, columns, visible_rows, visible_item_count);
    if hidden == 0 {
        return 0.0;
    }
    (first_row as f32 / hidden as f32).clamp(0.0, 1.0)
}

pub fn first_row_for_offset(hidden: i32, offset: f32) -> i32 {
    (((offset * hidden as f32) as f64 + 0.5) as i32).max(0)
}

pub fn first_item_index_for_offset(
    offset: f32,
    item_count: i32,
    columns: i32,
    visible_rows: i32,
    visible_item_count: i32,
) -> i32 {
    let hidden = hidden_rows(item_count, columns, visible_rows, visible_item_count);
    let first_row = first_row_for_offset(hidden, offset);
    let first = if item_count == 0 || columns == 0 {
        0
    } else {
        first_row.max(0) * columns
    };
    let last = max_first_item_index(item_count, columns, visible_rows, visible_item_count);
    if offset >= 1.0 { last } else { first.min(last) }
}

pub fn smooth_scroll_pixel_offset(hidden: i32, row_height: i32, offset: f32) -> i32 {
    let pixels = hidden * row_height;
    ((pixels as f32 * offset).round() as i32).clamp(0, pixels)
}

pub fn hidden_scroll_pixels(hidden: i32, row_height: i32, visible_height: i32) -> i32 {
    if row_height <= 0 {
        return 0;
    }
    let partial = visible_height.max(0) % row_height;
    (hidden * row_height - partial).max(0)
}

pub fn first_row_for_pixel_offset(pixels: i32, row_height: i32) -> i32 {
    if row_height == 0 {
        0
    } else {
        pixels / row_height
    }
}

pub fn row_pixel_offset(pixels: i32, row_height: i32) -> i32 {
    if row_height == 0 {
        0
    } else {
        pixels % row_height
    }
}

#[allow(clippy::too_many_arguments)]
pub fn offset_keeping_anchor_visible(
    anchor_index: i32,
    item_count: i32,
    columns: i32,
    visible_rows: i32,
    visible_item_count: i32,
    anchor_position: f32,
    smooth: bool,
    row_height: i32,
    visible_height: i32,
) -> f32 {
    let hidden = hidden_rows(item_count, columns, visible_rows, visible_item_count);
    if anchor_index < 0 || hidden == 0 || columns == 0 || visible_rows == 0 {
        return 0.0;
    }
    let anchor_row = anchor_index / columns;
    let position = anchor_position.clamp(0.0, 1.0);
    if smooth {
        if row_height == 0 {
            return 0.0;
        }
        let pixels = hidden_scroll_pixels(hidden, row_height, visible_height);
        if pixels == 0 {
            return 0.0;
        }
        let top = anchor_row * row_height;
        let target = (position * visible_height as f32).round() as i32;
        let lowest = (top - (visible_height - row_height)).max(0);
        let highest = pixels.min(top);
        return (top - target).clamp(lowest, highest) as f32 / pixels as f32;
    }
    let target_row = ((position * visible_rows as f32).round() as i32).clamp(0, visible_rows - 1);
    let lowest = (anchor_row - visible_rows + 1).max(0);
    let highest = hidden.min(anchor_row);
    (anchor_row - target_row).clamp(lowest, highest) as f32 / hidden as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    // The eleven cases of JEI's GridScrollMathTest.
    fn hidden(items: i32, columns: i32, rows: i32) -> i32 {
        hidden_rows(items, columns, rows, columns * rows)
    }

    #[test]
    fn hidden_rows_are_total_rows_minus_visible_rows() {
        assert_eq!(total_rows(100, 9), 12);
        assert_eq!(hidden(100, 9, 6), 6);
    }

    #[test]
    fn hidden_rows_account_for_blocked_visible_slots() {
        assert_eq!(hidden_rows(50, 9, 6, 45), 1);
    }

    #[test]
    fn valid_scroll_offset_clamps_to_bottom() {
        assert_eq!(valid_scroll_offset(50.0, 100, 9, 6, 54), 1.0);
    }

    #[test]
    fn first_item_index_uses_rounded_scroll_offset() {
        let offset = 2.6 / hidden(100, 9, 6) as f32;
        assert_eq!(first_item_index_for_offset(offset, 100, 9, 6, 54), 27);
    }

    #[test]
    fn first_item_index_at_bottom_accounts_for_blocked_visible_slots() {
        assert_eq!(first_item_index_for_offset(1.0, 100, 9, 6, 45), 55);
    }

    #[test]
    fn smooth_scroll_offset_keeps_sub_row_pixels() {
        let offset = 6.0 / (2.0 * 18.0);
        let pixels = smooth_scroll_pixel_offset(2, 18, offset);
        assert_eq!(first_row_for_pixel_offset(pixels, 18), 0);
        assert_eq!(row_pixel_offset(pixels, 18), 6);
    }

    #[test]
    fn hidden_pixels_account_for_partial_visible_row() {
        assert_eq!(hidden_scroll_pixels(3, 18, 2 * 18 + 7), 3 * 18 - 7);
    }

    #[test]
    fn scroll_offset_keeping_anchor_visible_preserves_relative_row_position() {
        let offset = offset_keeping_anchor_visible(210, 1000, 10, 20, 200, 0.1, false, 18, 20 * 18);
        assert_eq!(first_row_for_offset(hidden(1000, 10, 20), offset), 19);
    }

    #[test]
    fn smooth_scroll_offset_keeping_anchor_visible_preserves_relative_pixel_position() {
        let offset = offset_keeping_anchor_visible(210, 1000, 10, 10, 100, 0.15, true, 10, 100);
        assert_eq!(
            smooth_scroll_pixel_offset(hidden(1000, 10, 10), 10, offset),
            195
        );
    }

    #[test]
    fn scroll_offset_keeping_anchor_visible_clamps_at_bottom() {
        let offset = offset_keeping_anchor_visible(990, 1000, 10, 10, 100, 0.1, false, 18, 180);
        assert_eq!(first_row_for_offset(hidden(1000, 10, 10), offset), 90);
    }

    #[test]
    fn first_item_index_is_zero_when_grid_has_no_columns() {
        assert_eq!(first_item_index_for_offset(2.0, 100, 0, 6, 0), 0);
    }

    #[test]
    fn the_offset_of_a_first_row_is_its_share_of_the_hidden_rows() {
        assert_eq!(offset_for_first_row(3, 100, 9, 6, 54), 0.5);
        assert_eq!(offset_for_first_row(3, 10, 9, 6, 54), 0.0);
    }
}
