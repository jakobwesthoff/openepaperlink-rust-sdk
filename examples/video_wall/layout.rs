//! Grid geometry: where each tag's image is cut from the combined canvas.
//!
//! The wall is a `rows × cols` grid of identical tags. Each tag contributes a
//! `padding`-pixel bezel border on every side, representing the physical gap
//! between panels. In the virtual image each tag therefore owns a
//! `(cell + 2·padding)` cell, with its active display centered inside; the
//! border falls "on the bezel" and is skipped. With `padding == 0` the cells
//! tile edge to edge.

/// A rectangular region in canvas pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    /// Left edge.
    pub x: u32,
    /// Top edge.
    pub y: u32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

/// The wall layout.
#[derive(Debug, Clone, Copy)]
pub struct Grid {
    /// Number of tag rows.
    pub rows: u32,
    /// Number of tag columns.
    pub cols: u32,
    /// Active display width of one tag, in pixels.
    pub cell_width: u32,
    /// Active display height of one tag, in pixels.
    pub cell_height: u32,
    /// Bezel border around every tag, in pixels.
    pub padding: u32,
}

impl Grid {
    /// Outer cell size (active display plus the bezel on both sides).
    fn cell_size(&self) -> (u32, u32) {
        (
            self.cell_width + 2 * self.padding,
            self.cell_height + 2 * self.padding,
        )
    }

    /// Size of the virtual canvas the source image is scaled to cover.
    pub fn canvas_size(&self) -> (u32, u32) {
        let (cw, ch) = self.cell_size();
        (self.cols * cw, self.rows * ch)
    }

    /// The active display rectangle for the tag at (`row`, `col`) — the crop of
    /// the canvas that becomes that tag's image. Excludes the bezel border.
    pub fn tile_rect(&self, row: u32, col: u32) -> Rect {
        let (cw, ch) = self.cell_size();
        Rect {
            x: col * cw + self.padding,
            y: row * ch + self.padding,
            width: self.cell_width,
            height: self.cell_height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_size_without_padding_is_plain_tiling() {
        let grid = Grid { rows: 2, cols: 3, cell_width: 296, cell_height: 128, padding: 0 };
        assert_eq!(grid.canvas_size(), (3 * 296, 2 * 128));
    }

    #[test]
    fn canvas_size_includes_padding_on_both_sides() {
        let grid = Grid { rows: 1, cols: 2, cell_width: 100, cell_height: 50, padding: 10 };
        // Each cell is 120×70; two columns wide, one row tall.
        assert_eq!(grid.canvas_size(), (2 * 120, 70));
    }

    #[test]
    fn tile_rect_without_padding_tiles_edge_to_edge() {
        let grid = Grid { rows: 1, cols: 2, cell_width: 100, cell_height: 50, padding: 0 };
        assert_eq!(grid.tile_rect(0, 0), Rect { x: 0, y: 0, width: 100, height: 50 });
        assert_eq!(grid.tile_rect(0, 1), Rect { x: 100, y: 0, width: 100, height: 50 });
    }

    #[test]
    fn tile_rect_offsets_active_area_by_padding() {
        let grid = Grid { rows: 2, cols: 2, cell_width: 100, cell_height: 50, padding: 10 };
        // Cell size 120×70. Tile (1,1) starts at cell (120,70) plus padding (10,10).
        assert_eq!(
            grid.tile_rect(1, 1),
            Rect { x: 120 + 10, y: 70 + 10, width: 100, height: 50 },
        );
    }

    #[test]
    fn single_tile_rect_is_just_the_padding_offset() {
        let grid = Grid { rows: 1, cols: 1, cell_width: 296, cell_height: 128, padding: 8 };
        assert_eq!(grid.tile_rect(0, 0), Rect { x: 8, y: 8, width: 296, height: 128 });
        assert_eq!(grid.canvas_size(), (296 + 16, 128 + 16));
    }
}
