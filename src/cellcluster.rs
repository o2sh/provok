use crate::cell::Cell;

#[derive(Debug, Clone)]
pub struct CellCluster {
    pub text: String,
    pub byte_to_cell_idx: Vec<usize>,
}

impl CellCluster {
    pub fn make_cluster<'a>(iter: impl Iterator<Item = (usize, &'a Cell)>) -> Option<CellCluster> {
        let mut cluster = None;

        for (cell_idx, c) in iter {
            let cell_str = c.str();

            cluster = match cluster.take() {
                None => Some(CellCluster::new(cell_str, cell_idx)),
                Some(mut last) => {
                    last.add(cell_str, cell_idx);
                    Some(last)
                }
            };
        }

        cluster
    }

    fn new(text: &str, cell_idx: usize) -> CellCluster {
        let mut idx = Vec::new();
        for _ in 0..text.len() {
            idx.push(cell_idx);
        }
        CellCluster { text: text.into(), byte_to_cell_idx: idx }
    }

    fn add(&mut self, text: &str, cell_idx: usize) {
        for _ in 0..text.len() {
            self.byte_to_cell_idx.push(cell_idx);
        }
        self.text.push_str(text);
    }
}
