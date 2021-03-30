use crate::cell::{Cell, CellAttributes};
use crate::cellcluster::CellCluster;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    cells: Vec<Cell>,
}

impl Line {
    pub fn from_text(s: &str, attrs: &CellAttributes) -> Line {
        let mut cells = Vec::new();

        for sub in s.graphemes(true) {
            let cell = Cell::new_grapheme(sub, attrs.clone());
            let width = cell.width();
            cells.push(cell);
            for _ in 1..width {
                cells.push(Cell::new(' ', attrs.clone()));
            }
        }

        Line { cells }
    }

    pub fn visible_cells(&self) -> impl Iterator<Item = (usize, &Cell)> {
        let mut skip_width = 0;
        self.cells.iter().enumerate().filter(move |(_idx, cell)| {
            if skip_width > 0 {
                skip_width -= 1;
                false
            } else {
                skip_width = cell.width().saturating_sub(1);
                true
            }
        })
    }

    pub fn cluster(&self) -> Vec<CellCluster> {
        CellCluster::make_cluster(self.visible_cells())
    }
}

impl<'a> From<&'a str> for Line {
    fn from(s: &str) -> Line {
        Line::from_text(s, &CellAttributes::default())
    }
}
