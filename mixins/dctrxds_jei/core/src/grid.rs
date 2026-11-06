//! Where the panels go on a screen, in GUI pixels. The list goes right of the
//! screen's window and the bookmarks left of it, as JEI's overlays do.

/// One slot's side.
pub const SLOT: f32 = 18.0;
/// The gap to the screen's window and to the edges of the game window.
pub const MARGIN: f32 = 4.0;
/// The height of the page buttons' row above the slots.
pub const NAV: f32 = 16.0;
/// The height of the search box's row under the slots of the list.
pub const SEARCH: f32 = 18.0;
/// The most columns a panel takes, JEI's default.
pub const MAX_COLUMNS: usize = 9;
/// The fewest columns worth showing.
pub const MIN_COLUMNS: usize = 2;

/// Which screen the overlay is over.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenKind {
    Inventory,
    Creative,
}

/// What the overlay needs to know about the open screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenInfo {
    pub kind: ScreenKind,
    /// The screen's own window: `[x, y, width, height]`.
    pub rect: [f32; 4],
    /// The game window's width and height.
    pub window: [f32; 2],
    pub mouse: [f32; 2],
}

pub fn inside(rect: [f32; 4], point: [f32; 2]) -> bool {
    let [x, y, w, h] = rect;
    point[0] >= x && point[0] < x + w && point[1] >= y && point[1] < y + h
}

/// A panel's place on a screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid {
    /// All of the panel, buttons and search box included.
    pub area: [f32; 4],
    pub columns: usize,
    pub rows: usize,
    footer: f32,
}

impl Grid {
    fn in_columns(
        left: f32,
        right: f32,
        align_right: bool,
        screen: &ScreenInfo,
        footer: f32,
    ) -> Option<Self> {
        let height = screen.window[1] - 2.0 * MARGIN - NAV - footer;
        // A NaN or negative size casts to 0 and so has no room.
        let columns = (((right - left) / SLOT).floor() as usize).min(MAX_COLUMNS);
        let rows = (height / SLOT).floor() as usize;
        if columns < MIN_COLUMNS || rows == 0 {
            return None;
        }
        let width = columns as f32 * SLOT;
        let x = if align_right { right - width } else { left };
        Some(Self {
            area: [x, MARGIN, width, NAV + rows as f32 * SLOT + footer],
            columns,
            rows,
            footer,
        })
    }

    /// The list: right of the screen's window, with a search box under it.
    pub fn list(screen: &ScreenInfo) -> Option<Self> {
        let left = screen.rect[0] + screen.rect[2] + MARGIN;
        Self::in_columns(left, screen.window[0] - MARGIN, false, screen, SEARCH)
    }

    /// The bookmarks: left of the screen's window.
    pub fn bookmarks(screen: &ScreenInfo) -> Option<Self> {
        Self::in_columns(MARGIN, screen.rect[0] - MARGIN, true, screen, 0.0)
    }

    pub fn capacity(&self) -> usize {
        self.columns * self.rows
    }

    pub fn contains(&self, point: [f32; 2]) -> bool {
        inside(self.area, point)
    }

    /// The row of page buttons.
    pub fn nav(&self) -> [f32; 4] {
        [self.area[0], self.area[1], self.area[2], NAV]
    }

    /// The search box's row.
    pub fn footer(&self) -> [f32; 4] {
        let [x, y, w, h] = self.area;
        [x, y + h - self.footer + 2.0, w, self.footer - 2.0]
    }

    /// The top left corner of slot `index`, row by row.
    pub fn slot_position(&self, index: usize) -> [f32; 2] {
        let column = index % self.columns;
        let row = index / self.columns;
        [
            self.area[0] + column as f32 * SLOT,
            self.area[1] + NAV + row as f32 * SLOT,
        ]
    }

    /// The slot under `point`, if any.
    pub fn slot_at(&self, point: [f32; 2]) -> Option<usize> {
        let top = self.area[1] + NAV;
        if !self.contains(point) || point[1] < top {
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
            kind: ScreenKind::Inventory,
            rect: [152.0, 52.0, 176.0, 166.0],
            window: [480.0, 270.0],
            mouse: [0.0, 0.0],
        }
    }

    #[test]
    fn the_list_starts_right_of_the_screen_and_fits_the_window() {
        let grid = Grid::list(&inventory()).expect("room");
        assert_eq!(grid.area[0], 332.0);
        // (480 - 4 - 332) / 18 = 8 columns; (270 - 8 - 16 - 18) / 18 = 12 rows.
        assert_eq!((grid.columns, grid.rows), (8, 12));
        assert_eq!(grid.capacity(), 96);
        let [x, y, w, h] = grid.area;
        assert!(x + w <= 480.0 && y + h <= 270.0);
    }

    #[test]
    fn the_bookmarks_end_left_of_the_screen() {
        let grid = Grid::bookmarks(&inventory()).expect("room");
        let [x, _, w, _] = grid.area;
        assert_eq!(x + w, 148.0);
        assert!(x >= MARGIN);
        assert_eq!(grid.columns, 8);
    }

    #[test]
    fn neither_panel_covers_the_screen_at_any_gui_scale() {
        // A 1920 x 1080 window at GUI scales 1 to 4, and a tiny window.
        for (w, h) in [
            (1920.0, 1080.0),
            (960.0, 540.0),
            (640.0, 360.0),
            (480.0, 270.0),
            (200.0, 150.0),
        ] {
            let x = ((w - 176.0_f32) / 2.0).floor();
            let y = ((h - 166.0_f32) / 2.0).floor();
            let screen = ScreenInfo {
                rect: [x, y, 176.0, 166.0],
                window: [w, h],
                ..inventory()
            };
            for grid in [Grid::list(&screen), Grid::bookmarks(&screen)]
                .into_iter()
                .flatten()
            {
                let [gx, gy, gw, gh] = grid.area;
                assert!(gx + gw <= x || gx >= x + 176.0, "{w}x{h} covers the screen");
                assert!(gx >= 0.0 && gx + gw <= w && gy >= 0.0 && gy + gh <= h);
            }
        }
    }

    #[test]
    fn the_columns_stop_at_the_maximum() {
        let wide = ScreenInfo {
            window: [4000.0, 270.0],
            ..inventory()
        };
        assert_eq!(Grid::list(&wide).expect("room").columns, MAX_COLUMNS);
    }

    #[test]
    fn no_room_gives_no_grid() {
        let narrow = ScreenInfo {
            window: [360.0, 270.0],
            ..inventory()
        };
        assert_eq!(Grid::list(&narrow), None);
        let short = ScreenInfo {
            window: [480.0, 40.0],
            ..inventory()
        };
        assert_eq!(Grid::list(&short), None);
        let broken = ScreenInfo {
            window: [f32::NAN, f32::NAN],
            ..inventory()
        };
        assert_eq!(Grid::list(&broken), None);
    }

    #[test]
    fn slots_round_trip_through_their_positions() {
        let grid = Grid::list(&inventory()).expect("room");
        for index in 0..grid.capacity() {
            let [x, y] = grid.slot_position(index);
            assert_eq!(grid.slot_at([x, y]), Some(index));
            assert_eq!(grid.slot_at([x + 17.5, y + 17.5]), Some(index));
        }
    }

    #[test]
    fn the_rows_around_the_slots_and_the_outside_hit_no_slot() {
        let grid = Grid::list(&inventory()).expect("room");
        let [x, y, w, h] = grid.area;
        assert!(grid.contains([x, y]));
        assert_eq!(grid.slot_at([x, y]), None);
        assert_eq!(grid.slot_at([x - 1.0, y + NAV]), None);
        assert_eq!(grid.slot_at([x + w, y + NAV]), None);
        assert_eq!(grid.slot_at([x, y + h - 1.0]), None);
    }
}
