use pyo3::prelude::*;
use rstar::primitives::{GeomWithData, Rectangle};
use rstar::{RTree, AABB};

type SpatialNode = GeomWithData<Rectangle<[f64; 2]>, usize>;

#[pyclass]
#[derive(Clone, Debug)]
pub struct TextSpan {
    #[pyo3(get, set)]
    pub page: usize,
    #[pyo3(get, set)]
    pub x0: f64,
    #[pyo3(get, set)]
    pub y0: f64,
    #[pyo3(get, set)]
    pub x1: f64,
    #[pyo3(get, set)]
    pub y1: f64,
    #[pyo3(get, set)]
    pub text: String,
}

#[pymethods]
impl TextSpan {
    #[new]
    pub fn new(page: usize, x0: f64, y0: f64, x1: f64, y1: f64, text: String) -> Self {
        Self { page, x0, y0, x1, y1, text }
    }
}

#[pyclass]
pub struct SpatialTableIndex {
    spans: Vec<TextSpan>,
    tree: RTree<SpatialNode>,
    page_stride: f64,
}

#[pymethods]
impl SpatialTableIndex {
    #[new]
    #[pyo3(signature = (spans, page_stride=1000.0))]
    pub fn new(spans: Vec<TextSpan>, page_stride: f64) -> Self {
        let nodes: Vec<SpatialNode> = spans
            .iter()
            .enumerate()
            .map(|(idx, s)| {
                let offset_y = (s.page as f64) * page_stride;
                let rect = Rectangle::from_corners(
                    [s.x0, s.y0 + offset_y],
                    [s.x1, s.y1 + offset_y],
                );
                GeomWithData::new(rect, idx)
            })
            .collect();

        let tree = RTree::bulk_load(nodes);
        Self { spans, tree, page_stride }
    }

    /// Query all text spans intersecting a normalized bounding region across pages
    pub fn query_region(&self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Vec<TextSpan> {
        let envelope = AABB::from_corners([min_x, min_y], [max_x, max_y]);
        self.tree
            .locate_in_envelope_intersecting(&envelope)
            .map(|node| self.spans[node.data].clone())
            .collect()
    }

    /// Stitch broken table rows across page boundaries via X-column projection & Y-band clustering
    #[pyo3(signature = (y_tol=4.5, x_col_tol=18.0))]
    pub fn reconstruct_grid(&self, y_tol: f64, x_col_tol: f64) -> Vec<Vec<String>> {
        if self.spans.is_empty() {
            return Vec::new();
        }

        // 1. Project spans onto continuous virtual Y-axis
        let mut items: Vec<(f64, f64, &str)> = self
            .spans
            .iter()
            .map(|s| {
                let vy = (s.page as f64) * self.page_stride + (s.y0 + s.y1) * 0.5;
                let vx = (s.x0 + s.x1) * 0.5;
                (vy, vx, s.text.as_str())
            })
            .collect();

        items.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // 2. Discover canonical column anchors via 1D clustering on X centers
        let mut xs: Vec<f64> = items.iter().map(|i| i.1).collect();
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mut col_anchors: Vec<f64> = Vec::new();
        for x in xs {
            if let Some(last) = col_anchors.last_mut() {
                if (x - *last).abs() <= x_col_tol {
                    *last = (*last + x) * 0.5;
                    continue;
                }
            }
            col_anchors.push(x);
        }

        // 3. Group into horizontal row bands
        let mut rows: Vec<Vec<(f64, &str)>> = Vec::new();
        let mut current_y = items[0].0;
        let mut current_row: Vec<(f64, &str)> = Vec::new();

        for (vy, vx, text) in items {
            if (vy - current_y).abs() <= y_tol {
                current_row.push((vx, text));
            } else {
                rows.push(current_row);
                current_row = vec![(vx, text)];
                current_y = vy;
            }
        }
        if !current_row.is_empty() {
            rows.push(current_row);
        }

        // 4. Align cells to nearest column anchor
        let num_cols = col_anchors.len();
        rows.into_iter()
            .map(|row| {
                let mut grid_row = vec![String::new(); num_cols];
                for (vx, text) in row {
                    let mut best_col = 0;
                    let mut best_dist = f64::MAX;
                    for (c_idx, &anchor) in col_anchors.iter().enumerate() {
                        let d = (vx - anchor).abs();
                        if d < best_dist {
                            best_dist = d;
                            best_col = c_idx;
                        }
                    }
                    if grid_row[best_col].is_empty() {
                        grid_row[best_col] = text.to_string();
                    } else {
                        grid_row[best_col].push(' ');
                        grid_row[best_col].push_str(text);
                    }
                }
                grid_row
            })
            .collect()
    }
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<TextSpan>()?;
    m.add_class::<SpatialTableIndex>()?;
    Ok(())
}
