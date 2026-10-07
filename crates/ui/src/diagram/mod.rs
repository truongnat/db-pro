pub mod design_mode;
pub mod layout;
pub mod lod;
pub mod model;
pub mod persistence;
pub mod scene;
pub mod spatial;
pub mod viewport;

#[cfg(test)]
mod tests;

pub use layout::{ErLayoutRequest, ErLayoutResult, ErLayoutState, ErLayoutWorker};
pub use lod::ErLod;
pub use model::{
    er_table_key, ErEdge, ErGraph, ErNode, ER_CANVAS_MARGIN, ER_GAP_X, ER_GAP_Y, ER_HEADER_HEIGHT,
    ER_LARGE_SCHEMA_THRESHOLD, ER_MAX_COLUMNS, ER_MAX_EDGES, ER_MAX_TABLES, ER_MAX_ZOOM, ER_MIN_ZOOM, ER_NODE_WIDTH,
    ER_ROW_HEIGHT,
};
pub use persistence::{ErLayoutSnapshot, ER_LAYOUTS_STORAGE_KEY};
pub use scene::{prepare_render_scene, ErPerfMetrics, ErRenderScene};
pub use spatial::{ErSpatialIndex, ErSpatialMetrics, DEFAULT_SPATIAL_CELL_SIZE};
pub use viewport::ErViewport;
