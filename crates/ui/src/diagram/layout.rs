use super::model::ErGraph;
use super::spatial::{ErSpatialIndex, DEFAULT_SPATIAL_CELL_SIZE};
use crate::runtime::UiTableSummary;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErLayoutState {
    Idle,
    Computing { request_id: u64, graph_version: u64 },
    Ready,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct ErLayoutRequest {
    pub request_id: u64,
    pub graph_version: u64,
    pub tables: Vec<UiTableSummary>,
    pub grid_columns: usize,
    pub node_height: f32,
}

#[derive(Debug, Clone)]
pub struct ErLayoutResult {
    pub request_id: u64,
    pub graph_version: u64,
    pub graph: ErGraph,
    pub spatial_index: ErSpatialIndex,
    pub duration_ms: f32,
}

pub struct ErLayoutWorker {
    request_tx: Sender<ErLayoutRequest>,
    result_rx: Receiver<ErLayoutResult>,
    next_request_id: u64,
}

impl Default for ErLayoutWorker {
    fn default() -> Self {
        Self::new()
    }
}

impl ErLayoutWorker {
    pub fn new() -> Self {
        let (request_tx, request_rx) = mpsc::channel::<ErLayoutRequest>();
        let (result_tx, result_rx) = mpsc::channel::<ErLayoutResult>();

        thread::Builder::new()
            .name("db-pro-er-layout".to_owned())
            .spawn(move || {
                while let Ok(request) = request_rx.recv() {
                    // Drain any newer pending requests to coalesce and prioritize the latest schema
                    let mut latest_request = request;
                    while let Ok(newer) = request_rx.try_recv() {
                        latest_request = newer;
                    }

                    let start_time = Instant::now();
                    let graph = ErGraph::build(
                        &latest_request.tables,
                        latest_request.graph_version,
                        latest_request.grid_columns,
                        latest_request.node_height,
                    );
                    let spatial_index = ErSpatialIndex::build(&graph.nodes, &graph.edges, DEFAULT_SPATIAL_CELL_SIZE);
                    let duration_ms = start_time.elapsed().as_secs_f32() * 1000.0;

                    let _ = result_tx.send(ErLayoutResult {
                        request_id: latest_request.request_id,
                        graph_version: latest_request.graph_version,
                        graph,
                        spatial_index,
                        duration_ms,
                    });
                }
            })
            .expect("failed to spawn er layout background worker thread");

        Self {
            request_tx,
            result_rx,
            next_request_id: 1,
        }
    }

    pub fn request_layout(
        &mut self,
        graph_version: u64,
        tables: Vec<UiTableSummary>,
        grid_columns: usize,
        node_height: f32,
    ) -> u64 {
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.saturating_add(1);

        let _ = self.request_tx.send(ErLayoutRequest {
            request_id,
            graph_version,
            tables,
            grid_columns,
            node_height,
        });

        request_id
    }

    pub fn poll_result(&self) -> Option<ErLayoutResult> {
        let mut latest = None;
        while let Ok(res) = self.result_rx.try_recv() {
            latest = Some(res);
        }
        latest
    }
}
