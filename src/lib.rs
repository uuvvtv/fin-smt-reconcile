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
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<TextSpan>()?;
    m.add_class::<SpatialTableIndex>()?;
    Ok(())
}
