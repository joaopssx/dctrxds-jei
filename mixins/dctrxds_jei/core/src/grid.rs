//! Where the item grid goes on a screen, in GUI pixels.

/// One slot's side.
pub const SLOT: f32 = 18.0;
/// The gap to the screen's window and to the edges of the game window.
pub const MARGIN: f32 = 4.0;
/// The height of the text rows above the slots.
pub const HEADER: f32 = 40.0;
/// The most columns the grid takes.
pub const MAX_COLUMNS: usize = 9;
/// The fewest columns worth showing.
pub const MIN_COLUMNS: usize = 2;

/// What the grid needs to know about the open screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenInfo {
    /// The screen's own window: `[x, y, width, height]`.
    pub rect: [f32; 4],
    /// The game window's width and height.
    pub window: [f32; 2],
    pub mouse: [f32; 2],
}

/// The grid's place on a screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid {
    /// Everything that belongs to the overlay, text rows included:
    /// `[x, y, width, height]`.
    pub area: [f32; 4],
    pub columns: usize,
    pub rows: usize,
}

impl Grid {
    /// The grid right of the screen's window, or `None` when there is no room.
    pub fn compute(screen: &ScreenInfo) -> Option<Self> {
        let left = screen.rect[0] + screen.rect[2] + MARGIN;
        let width = screen.window[0] - MARGIN - left;
        let height = screen.window[1] - 2.0 * MARGIN - HEADER;
        // A NaN or negative size casts to 0 and so has no room.
        let columns = ((width / SLOT).floor() as usize).min(MAX_COLUMNS);
        let rows = (height / SLOT).floor() as usize;
        if columns < MIN_COLUMNS || rows == 0 {
            return None;
        }
        Some(Self {
            area: [
                left,
                MARGIN,
                columns as f32 * SLOT,
                HEADER + rows as f32 * SLOT,
            ],
            columns,
            rows,
        })
    }

    pub fn capacity(&self) -> usize {
        self.columns * self.rows
    }

    pub fn contains(&self, point: [f32; 2]) -> bool {
        let [x, y, w, h] = self.area;
        point[0] >= x && point[0] < x + w && point[1] >= y && point[1] < y + h
    }

    /// The top left corner of slot `index`, row by row.
    pub fn slot_position(&self, index: usize) -> [f32; 2] {
        let column = index % self.columns;
        let row = index / self.columns;
        [
            self.area[0] + column as f32 * SLOT,
            self.area[1] + HEADER + row as f32 * SLOT,
        ]
    }

    /// The slot under `point`, if any.
    pub fn slot_at(&self, point: [f32; 2]) -> Option<usize> {
        if !self.contains(point) {
            return None;
        }
        let top = self.area[1] + HEADER;
        if point[1] < top {
            return None;
        }
        let column = ((point[0] - self.area[0]) / SLOT) as usize;
        let row = ((point[1] - top) / SLOT) as usize;
        (column < self.columns && row < self.rows).then_some(row * self.columns + column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 176 wide inventory centered in a 480 x 270 GUI window.
    fn inventory() -> ScreenInfo {
        ScreenInfo {
            rect: [152.0, 52.0, 176.0, 166.0],
            window: [480.0, 270.0],
            mouse: [0.0, 0.0],
        }
    }

    #[test]
    fn the_grid_starts_right_of_the_screen_and_fits_the_window() {
        let grid = Grid::compute(&inventory()).expect("room");
        assert_eq!(grid.area[0], 332.0);
        // (480 - 4 - 332) / 18 = 8 columns; (270 - 8 - 40) / 18 = 12 rows.
        assert_eq!((grid.columns, grid.rows), (8, 12));
        assert_eq!(grid.capacity(), 96);
        let [x, y, w, h] = grid.area;
        assert!(x + w <= 480.0 && y + h <= 270.0);
    }

    #[test]
    fn the_columns_stop_at_the_maximum() {
        let wide = ScreenInfo {
            window: [4000.0, 270.0],
            ..inventory()
        };
        assert_eq!(Grid::compute(&wide).expect("room").columns, MAX_COLUMNS);
    }

    #[test]
    fn no_room_gives_no_grid() {
        let narrow = ScreenInfo {
            window: [360.0, 270.0],
            ..inventory()
        };
        assert_eq!(Grid::compute(&narrow), None);
        let short = ScreenInfo {
            window: [480.0, 60.0],
            ..inventory()
        };
        assert_eq!(Grid::compute(&short), None);
        let broken = ScreenInfo {
            window: [f32::NAN, f32::NAN],
            ..inventory()
        };
        assert_eq!(Grid::compute(&broken), None);
    }

    #[test]
    fn slots_round_trip_through_their_positions() {
        let grid = Grid::compute(&inventory()).expect("room");
        for index in 0..grid.capacity() {
            let [x, y] = grid.slot_position(index);
            assert_eq!(grid.slot_at([x, y]), Some(index));
            assert_eq!(grid.slot_at([x + 17.5, y + 17.5]), Some(index));
        }
    }

    #[test]
    fn the_text_rows_and_the_outside_hit_no_slot() {
        let grid = Grid::compute(&inventory()).expect("room");
        let [x, y, w, h] = grid.area;
        assert!(grid.contains([x, y]));
        assert_eq!(grid.slot_at([x, y]), None);
        assert_eq!(grid.slot_at([x - 1.0, y + HEADER]), None);
        assert_eq!(grid.slot_at([x + w, y + HEADER]), None);
        assert_eq!(grid.slot_at([x, y + h]), None);
    }
}
