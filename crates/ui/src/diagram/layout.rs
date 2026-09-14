use super::model::ErGraph;
use super::spatial::{ErSpatialIndex, DEFAULT_SPATIAL_CELL_SIZE};
use crate::runtime::UiTableSummary;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Instant;

/// Maximum number of in-flight coalesced requests the worker will drain
/// before processing. Prevents unbounded memory growth under rapid refresh.
const MAX_COALESCE_DRAIN: usize = 64;

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
    request_tx: Option<Sender<ErLayoutRequest>>,
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

        let worker_result = thread::Builder::new()
            .name("db-pro-er-layout".to_owned())
            .spawn(move || {
                while let Ok(request) = request_rx.recv() {
                    // Drain newer pending requests to coalesce and prioritize the latest schema.
                    // Cap the drain count to prevent unbounded CPU usage under rapid refresh.
                    let mut latest_request = request;
                    let mut drain_count = 0;
                    while drain_count < MAX_COALESCE_DRAIN {
                        match request_rx.try_recv() {
                            Ok(newer) => {
                                latest_request = newer;
                                drain_count += 1;
                            }
                            Err(TryRecvError::Empty) => break,
                            Err(TryRecvError::Disconnected) => break,
                        }
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

                    // If the receiver has been dropped (app shutting down), this send fails
                    // silently — the worker thread exits naturally on the next recv() error.
                    let _ = result_tx.send(ErLayoutResult {
                        request_id: latest_request.request_id,
                        graph_version: latest_request.graph_version,
                        graph,
                        spatial_index,
                        duration_ms,
                    });
                }
            });

        // If thread spawn fails (extremely rare: resource exhaustion), the worker
        // operates in degraded mode: request_layout silently drops requests,
        // poll_result never returns a result, and is_alive() returns false.
        // The UI retains the last valid graph.
        let request_tx = if worker_result.is_err() {
            eprintln!("[db-pro] Failed to spawn ER layout worker thread — degraded mode");
            drop(request_tx); // close the sending half so the receiver also sees disconnect
            None
        } else {
            Some(request_tx)
        };

        Self {
            request_tx,
            result_rx,
            next_request_id: 1,
        }
    }

    /// Returns `true` if the background worker thread is alive and the
    /// sending channel is connected.
    pub fn is_alive(&self) -> bool {
        self.request_tx.is_some()
    }

    pub fn request_layout(
        &mut self,
        graph_version: u64,
        tables: Vec<UiTableSummary>,
        grid_columns: usize,
        node_height: f32,
    ) -> u64 {
        let request_id = self.next_request_id;
        // Use saturating_add but cap at MAX - 1 so that the next call still
        // gets a distinct ID. At u64::MAX, request_id stays MAX and the
        // caller must know that duplicate IDs indicate a saturated counter.
        self.next_request_id = self.next_request_id.saturating_add(1);

        if let Some(tx) = &self.request_tx {
            let _ = tx.send(ErLayoutRequest {
                request_id,
                graph_version,
                tables,
                grid_columns,
                node_height,
            });
        }

        request_id
    }

    /// Returns `true` if a request was actually dispatched to the worker.
    /// Returns `false` in degraded mode (spawn failure) where requests are
    /// silently dropped.
    pub fn dispatch_succeeded(&self) -> bool {
        self.request_tx.is_some()
    }

    /// Drain all pending results and return the most recent one.
    /// This guarantees the caller always gets the latest completed computation,
    /// which is critical for stale-result correctness.
    pub fn poll_result(&self) -> Option<ErLayoutResult> {
        let mut latest = None;
        while let Ok(res) = self.result_rx.try_recv() {
            latest = Some(res);
        }
        latest
    }
}
