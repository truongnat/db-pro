import {
  lazy,
  Suspense,
  useCallback,
  useEffect,
  useLayoutEffect,
  useMemo,
  useReducer,
  useRef,
  useState,
} from "react";
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  type Node,
  type Edge,
  type Viewport,
  type ReactFlowInstance,
  MarkerType,
  BackgroundVariant,
  Panel,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { useTranslation } from "@/commons/locales/useTranslation";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { Search, Maximize2, LayoutGrid, Columns2, Table2, RotateCcw, Loader2 } from "lucide-react";

import { ErTableNode, type TableNodeData } from "./lod/er-table-node";
import { ErPerfHud } from "./er-perf-hud";
import { ErSearchEntry } from "./er-search-entry";
import {
  initialLargeSchemaState,
  largeSchemaReducer,
  shouldEnterLargeSchemaFlow,
  deriveNeighborhoodVisibleSet,
} from "../utils/large-schema";
import {
  buildErGraphModel,
  classifySchemaComplexity,
  computeSchemaComplexity,
  type SchemaComplexityTier,
} from "../renderer/er-graph-model";
import type { ErGraphModel, ErViewport } from "../renderer/types";
import { useWorkerLayout } from "../hooks/use-worker-layout";
import {
  layoutNodeHeight,
  LAYOUT_NODE_WIDTH,
  type LayoutInput,
  type LayoutPosition,
} from "../utils/layout";
import {
  buildLayoutInputFromModel,
  OVERVIEW_LAYOUT_PROFILE,
  REACT_FLOW_LAYOUT_PROFILE,
} from "../utils/layout-profile";
import { groupForeignKeys } from "../utils/edge-builder";
import type { NeighborhoodScope } from "../utils/neighborhood";
import { resolveHopCount } from "../utils/neighborhood";
import { ErPerfMonitor } from "../utils/instrumentation";
import { resolveLod, type LodLevel } from "../utils/lod";
import { aggregateRelations, resolveEdgeLod, type EdgeLodLevel } from "../utils/edge-lod";
import { SpatialIndex } from "../utils/spatial-index";

import type { IntrospectResult } from "@/modules/schema/types/schema.types";
import { buildErNodeIndexes, buildTableNodes } from "../renderer/er-node-builder";

interface ErDiagramProps {
  connectionId: string;
  schema: string;
  data: IntrospectResult;
}

const nodeTypes = { table: ErTableNode };

// Code-split the canvas overview: cytoscape (~450 kB min) only downloads when
// the user explicitly opens a large schema's "All N tables" overview — the
// initial bundle and the React Flow path stay lean.
const CytoscapeErView = lazy(() =>
  import("./cytoscape-view").then((m) => ({ default: m.CytoscapeErView })),
);

/** Build a storage key for persisting node positions per connection + schema. */
function positionStorageKey(connectionId: string, schemaName: string) {
  return `er-diagram-positions:${connectionId}:${schemaName}`;
}

/** Load cached manual positions from localStorage for a given identity. */
function loadPositions(
  connectionId: string,
  schema: string,
): Map<string, { x: number; y: number }> {
  try {
    const raw = localStorage.getItem(positionStorageKey(connectionId, schema));
    if (raw) return new Map(JSON.parse(raw));
  } catch {
    /* ignore */
  }
  return new Map();
}

export function ErDiagram({ connectionId, schema, data }: ErDiagramProps) {
  const { t } = useTranslation();
  const reactFlowRef = useRef<HTMLDivElement>(null);
  // F-MR-1: the mounted React Flow instance (captured in onInit) — fit-view
  // goes through `instance.fitView()` instead of the old synthetic `keydown
  // "1"` dispatch, which depended on React Flow's default shortcut.
  const rfInstanceRef = useRef<ReactFlowInstance | null>(null);

  // P1.1 runtime instrumentation — dev-only, enabled via localStorage `er-perf-hud=1`.
  const perfEnabled =
    typeof localStorage !== "undefined" && localStorage.getItem("er-perf-hud") === "1";
  const perfMonitorRef = useRef<ErPerfMonitor | null>(null);
  if (!perfMonitorRef.current) {
    perfMonitorRef.current = new ErPerfMonitor(() => reactFlowRef.current);
  }
  // Viewport lives in a ref (not state) — onViewportChange fires every pan/zoom
  // frame, and a setState per frame would defeat the purpose of this perf work.
  // The HUD refreshes on its own 500ms tick and reads the ref.
  const viewportRef = useRef<Viewport | null>(null);
  // Captured at first render so time-to-interactive is measured from mount,
  // not from a fixed setTimeout.
  const renderStartRef = useRef(performance.now());

  // Start the long-task observer and record mount time (dev-only).
  useEffect(() => {
    if (!perfEnabled) return;
    perfMonitorRef.current?.start();
    return () => perfMonitorRef.current?.stop();
  }, [perfEnabled]);

  const [compact, setCompact] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [layoutDirection, setLayoutDirection] = useState<"LR" | "TB">("LR");
  const [selectedEdgeId, setSelectedEdgeId] = useState<string | null>(null);
  const [currentLod, setCurrentLod] = useState<LodLevel>("detail");
  const [currentEdgeLod, setCurrentEdgeLod] = useState<EdgeLodLevel>("full");

  // Mirror `compact` into a ref so onViewportChange can read it without
  // re-creating the callback on every toggle.
  const compactRef = useRef(compact);
  compactRef.current = compact;

  const onViewportChange = useCallback((viewport: Viewport) => {
    viewportRef.current = viewport;
    setCurrentLod((prev) => {
      const next = resolveLod(viewport.zoom, compactRef.current);
      return next === prev ? prev : next;
    });
    setCurrentEdgeLod((prev) => {
      const next = resolveEdgeLod(viewport.zoom);
      return next === prev ? prev : next;
    });
  }, []);

  // Keep the compact toggle live even without a new viewport event.
  useEffect(() => {
    setCurrentLod((prev) => {
      const next = resolveLod(viewportRef.current?.zoom ?? 1, compact);
      return next === prev ? prev : next;
    });
  }, [compact]);

  // Schema tables memoized on the atomic introspection snapshot so dependent
  // memos (schemaStats) keep stable identities (pre-merge review P2 fix).
  const tablesInSchema = useMemo(
    () => data.tables.filter((t) => t.schema === schema),
    [data.tables, schema],
  );

  // P1.9 — renderer-agnostic graph model, shared by the Cytoscape overview.
  const graphModel = useMemo<ErGraphModel>(() => buildErGraphModel(data, schema), [data, schema]);

  // 6.11 — thresholds are complexity scores, not hardcoded table counts (locked
  // P1 hard rule #5): complexity = tables + relations*0.7 + columns*0.08. Tier
  // boundaries tuned from the P1.8 benchmark — A100(310.4)→M (full React Flow
  // graph at 60 fps, no exploration UX), A500(1,802.5)→L, A1000(3,765.2)→XL.
  const schemaComplexity = useMemo(() => computeSchemaComplexity(graphModel.stats), [graphModel]);
  const schemaTier = useMemo<SchemaComplexityTier>(
    () => classifySchemaComplexity(schemaComplexity),
    [schemaComplexity],
  );
  const isLargeSchema = schemaTier === "L" || schemaTier === "XL";

  // Gate 4 Slice B — search-first entry for large schemas.
  // Entry predicate (locked): tableCount > 200 OR tier L/XL.
  const useLargeSchemaFlow = shouldEnterLargeSchemaFlow(tablesInSchema.length, schemaTier);
  const [largeSchemaState, dispatchLargeSchema] = useReducer(
    largeSchemaReducer,
    initialLargeSchemaState,
  );

  // Gate 4 C4 (#40) — render-phase identity gate.
  // Detect connection/schema change BEFORE any derivation. When the identity
  // has changed, all downstream computations (phase, seed, focus, neighborhood,
  // layout input) use the initial state — never the stale state from the
  // previous identity. The useLayoutEffect below then commits the real reset.
  const prevConnectionRef = useRef(connectionId);
  const prevSchemaRef = useRef(schema);
  const identityChanged =
    prevConnectionRef.current !== connectionId || prevSchemaRef.current !== schema;
  const effectiveLargeSchemaState = identityChanged ? initialLargeSchemaState : largeSchemaState;

  const isSearchPhase = useLargeSchemaFlow && effectiveLargeSchemaState.phase === "search";
  const isNeighborhoodPhase =
    useLargeSchemaFlow && effectiveLargeSchemaState.phase === "neighborhood";
  // #45 E2 — explicit overview phase predicate.
  // Replaces the old `isLargeSchema && !isNeighborhoodPhase` which was
  // semantically too broad (true in search too, though masked by the
  // search-first render branch). Only `overview` activates Cytoscape
  // and the full-schema layout pipeline.
  const isOverviewPhase = useLargeSchemaFlow && effectiveLargeSchemaState.phase === "overview";

  const fittedPositionsRef = useRef<Map<string, LayoutPosition> | null>(null);

  // Persisted manual positions: nodeId → { x, y }
  const [manualPositions, setManualPositions] = useState<Map<string, { x: number; y: number }>>(
    () => loadPositions(connectionId, schema),
  );

  // Gate 4 C4 (#40) — lifecycle/reset.
  // The render-phase gate above (effectiveLargeSchemaState) ensures no stale
  // state leaks into memos/layout during the first render of a new identity.
  // This useLayoutEffect commits the real state reset before the browser paints.
  const phase = effectiveLargeSchemaState.phase;
  const prevPhaseRef = useRef(phase);

  useLayoutEffect(() => {
    if (identityChanged) {
      // Commit refs to new identity.
      prevConnectionRef.current = connectionId;
      prevSchemaRef.current = schema;
      prevPhaseRef.current = "search";

      // Reset reducer state to search.
      dispatchLargeSchema({ type: "BACK_TO_SEARCH" });
      setSearchQuery("");
      setSelectedEdgeId(null);
      fittedPositionsRef.current = null;

      // Load the NEW identity's cached positions (not empty Map).
      setManualPositions(loadPositions(connectionId, schema));
      return;
    }

    const phaseChanged = prevPhaseRef.current !== phase;
    if (!phaseChanged) return;
    prevPhaseRef.current = phase;
    setSearchQuery("");
    setSelectedEdgeId(null);
    fittedPositionsRef.current = null;
    if (phase === "neighborhood") {
      // Gate 4 D1 (#41) — safe first-paint LOD. Neighborhood mounts ≤100
      // nodes; starting at "detail" would mount full column lists before
      // fitView zooms out. "compact" is the cheapest readable level; the
      // viewport callback upgrades to the correct LOD after fitView.
      setCurrentLod("compact");
    }
    if (phase === "search") {
      setManualPositions(new Map());
      try {
        localStorage.setItem(positionStorageKey(connectionId, schema), "[]");
      } catch {
        /* ignore */
      }
    }
  }, [phase, connectionId, schema, identityChanged]);

  // UX pivot (opass.html): the FULL graph is the default experience for large
  // schemas — no landing screen, no neighborhood gate, no filter-first flow.
  // The hop scope below only sizes the neighborhood ring that search/click
  // focus draws on the always-visible canvas overview.
  // #46 E3 — neighborhood hops are fixed at 2 (not user-changeable).
  // Overview has its own separate `overviewHighlightHops` state.
  const neighborhoodHops: NeighborhoodScope = 2;

  // #46 E3 — separate hops state for overview highlight.
  // Overview's explorer control (1 hop / 2 hops / 3 hops / Domain) changes
  // ONLY the highlight ring on the canvas — it must NOT mutate the bounded
  // neighborhood context that is restored on Back navigation.
  const [overviewHighlightHops, setOverviewHighlightHops] = useState<NeighborhoodScope>(2);

  // Gate 4 C2 (#38) — bounded neighborhood materialization.
  // Derive the visible table set from #37's pure logic. Only tables in this
  // set get React Flow nodes/edges. Edges with hidden endpoints are dropped
  // by groupForeignKeys (both-endpoints check).
  //
  // `tableIds` preserves the deterministic ordering from #37 (canonical BFS
  // order). `keySet` is for O(1) edge-endpoint lookups.
  const neighborhoodVisible = useMemo<{ tableIds: string[]; keySet: Set<string> }>(() => {
    if (!isNeighborhoodPhase) return { tableIds: [], keySet: new Set<string>() };
    const knownTableKeys = new Set(tablesInSchema.map((t) => `${t.schema}.${t.name}`));
    const hops = resolveHopCount(neighborhoodHops);
    const result = deriveNeighborhoodVisibleSet(
      effectiveLargeSchemaState,
      graphModel.adjacency,
      hops,
      knownTableKeys,
    );
    return { tableIds: result.tableIds, keySet: new Set(result.tableIds) };
  }, [
    isNeighborhoodPhase,
    effectiveLargeSchemaState,
    graphModel.adjacency,
    tablesInSchema,
    neighborhoodHops,
  ]);

  // Gate 4 D2 (#42) — clear focus when the focused node leaves the visible set.
  // This happens when the user changes hop radius or seed and the previously
  // focused node is no longer in the bounded neighborhood.
  const focusedNodeId = effectiveLargeSchemaState.focusedNodeId;
  useEffect(() => {
    if (focusedNodeId && !neighborhoodVisible.keySet.has(focusedNodeId)) {
      dispatchLargeSchema({ type: "CLEAR_FOCUS" });
    }
  }, [focusedNodeId, neighborhoodVisible.keySet]);

  // Gate 4 C3 (#39) + #45 E2 — the canvas renderer (Cytoscape) and the full
  // overview layout pipeline are active ONLY in overview phase. Neighborhood
  // renders the bounded ≤100 graph on React Flow with column-aware geometry;
  // search renders the search entry. No other phase activates Cytoscape.
  const activeCytoscape = isOverviewPhase;

  // Schema statistics for the overview explorer.
  const schemaStats = useMemo(() => {
    const keys = new Set(tablesInSchema.map((t) => `${t.schema}.${t.name}`));
    let relations = 0;
    for (const fk of data.foreignKeys) {
      if (keys.has(`${fk.schema}.${fk.fromTable}`) && keys.has(`${fk.toSchema}.${fk.toTable}`)) {
        relations++;
      }
    }
    let columns = 0;
    for (const col of data.columns) {
      if (keys.has(`${col.schema}.${col.tableName}`)) columns++;
    }
    return { relations, columns };
  }, [data.foreignKeys, data.columns, tablesInSchema]);

  // Save positions to localStorage when they change
  const savePositions = useCallback(
    (positions: Map<string, { x: number; y: number }>) => {
      try {
        localStorage.setItem(
          positionStorageKey(connectionId, schema),
          JSON.stringify([...positions]),
        );
      } catch {
        /* ignore */
      }
    },
    [connectionId, schema],
  );

  // Pre-index metadata by schema.tableName — O(C + P + F) once, O(1) per table
  // lookup (P3.3). Pure builders in renderer/er-node-builder.ts so the real
  // component path is exercised by the unit/perf tests; memoized on `data`
  // (an atomic introspection snapshot) for stable identity.
  const nodeIndexes = useMemo(() => buildErNodeIndexes(data), [data]);

  // Build nodes and edges from introspection data using pre-indexed maps.
  // React Flow renders the full schema for small/medium schemas; large
  // schemas never reach this branch (canvas overview instead).
  //
  // Gate 4 Slice B + C2 (#38): skip entirely during search phase — no graph
  // nodes or edges are constructed before the user explicitly selects a table.
  // In neighborhood phase, only materialize nodes/edges for the bounded visible
  // set from #37 (≤100 tables). Edges with hidden endpoints are dropped.
  //
  // Fail-safe: neighborhood with empty visible set → 0 nodes / 0 edges.
  // Never falls back to the full schema.
  //
  // Deterministic order: nodes follow #37's canonical tableIds ordering,
  // not data.tables arrival order.
  const { initialNodes, initialEdges } = useMemo(() => {
    if (isSearchPhase) return { initialNodes: [] as Node[], initialEdges: [] as Edge[] };

    const allTables = data.tables.filter((tbl) => tbl.schema === schema);

    // #38 — in neighborhood phase, restrict to the bounded visible set.
    // Preserve #37's deterministic ordering via index lookup.
    // Empty visible set → 0 nodes (fail-safe, no fallback to allTables).
    let tables: typeof allTables;
    let visibleTableKeys: Set<string>;

    if (isNeighborhoodPhase) {
      const tableByKey = new Map(allTables.map((t) => [`${t.schema}.${t.name}`, t]));
      tables = neighborhoodVisible.tableIds
        .map((id) => tableByKey.get(id))
        .filter((t): t is (typeof allTables)[number] => t !== undefined);
      visibleTableKeys = neighborhoodVisible.keySet;
    } else {
      tables = allTables;
      visibleTableKeys = new Set(tables.map((t) => `${t.schema}.${t.name}`));
    }

    // Pure pre-indexed build (P3.3) — O(1) map lookups, no per-table scans
    // of data.columns / data.primaryKeys / data.foreignKeys.
    const nodes: Node[] = buildTableNodes(tables, nodeIndexes, { compact });

    const fkGroups = groupForeignKeys(data.foreignKeys, visibleTableKeys);

    const edges: Edge[] = fkGroups.map((group) => ({
      id: `fk:${group.key}`,
      source: `${group.fk.toSchema}.${group.fk.toTable}`,
      target: `${group.fk.schema}.${group.fk.fromTable}`,
      sourceHandle: `pk:${group.fk.toColumns[0]}`,
      targetHandle: `fk:${group.fk.fromColumns[0]}`,
      type: "smoothstep",
      animated: false,
      markerEnd: { type: MarkerType.ArrowClosed, width: 12, height: 12 },
      style: { strokeWidth: 1.5 },
      label: group.fk.name,
      labelStyle: { fontSize: 9, opacity: 0.6 },
    }));

    return { initialNodes: nodes, initialEdges: edges };
  }, [data, compact, schema, nodeIndexes, isSearchPhase, isNeighborhoodPhase, neighborhoodVisible]);

  // P1.7 — layout off the main thread.
  //
  // Build a plain (worker-serializable) layout input, run dagre in the Worker
  // (cache-first by content hash), and commit positions atomically when the
  // result lands. The UI stays interactive meanwhile and shows an "Arranging
  // N tables…" overlay (locked: no UNSTABLE positions — every commit is a
  // full, stable set; Option C adds complete force-refined stages, never
  // partial streams).
  //
  // P1-2 — the input geometry follows the RENDERER, not a single hardcoded
  // card size. The canvas overview lays out compact 160×28 nodes (its actual
  // paint geometry); React Flow lays out column-aware cards. The profile id
  // participates in the layout hash, so the two never share cached positions.
  // Gate 4 Slice B: skip the layout input builder during search phase.
  // Gate 4 C3 (#39): also skip during neighborhood — the bounded React Flow
  // path uses rfLayoutInput, so building the full-model overview input would
  // be wasted main-thread work for 500–1000-table schemas.
  // buildLayoutInputFromModel iterates every table/relation — for a 1000-table
  // schema that is non-trivial main-thread work on first paint.
  const overviewLayoutInput = useMemo<LayoutInput | null>(() => {
    if (isSearchPhase || !activeCytoscape || graphModel.tables.length === 0) return null;
    return buildLayoutInputFromModel(graphModel, OVERVIEW_LAYOUT_PROFILE);
  }, [graphModel, isSearchPhase, activeCytoscape]);

  const rfLayoutInput = useMemo<LayoutInput | null>(() => {
    if (isSearchPhase || initialNodes.length === 0) return null;
    return {
      nodes: initialNodes.map((n) => {
        const d = n.data as TableNodeData;
        return {
          id: n.id,
          height: layoutNodeHeight(d.columns?.length ?? 0, d.compact ?? false),
          width: LAYOUT_NODE_WIDTH,
        };
      }),
      edges: initialEdges.map((e) => ({ source: e.source, target: e.target })),
    };
  }, [initialNodes, initialEdges, isSearchPhase]);

  // #45 E2 / #46 E3 — layout input routing by phase:
  //   search      → null (no layout)
  //   neighborhood → rfLayoutInput (bounded ≤100 React Flow)
  //   overview    → overviewLayoutInput (full-schema Cytoscape)
  //
  // #46 E3 stale-position fix: TWO separate hook instances. Each has its own
  // internal state (positions, status). Overview positions can NEVER
  // contaminate React Flow — even during the computing transition when
  // Back is clicked and RF starts recomputing.
  const rfLayoutOptions = useMemo(
    () => ({ direction: layoutDirection, profile: REACT_FLOW_LAYOUT_PROFILE.id }),
    [layoutDirection],
  );
  const overviewLayoutOptions = useMemo(
    () => ({ direction: layoutDirection, profile: OVERVIEW_LAYOUT_PROFILE.id }),
    [layoutDirection],
  );

  // React Flow neighborhood layout — active in neighborhood phase only.
  // Passing `null` in search/overview prevents any layout computation.
  const rfLayout = useWorkerLayout(isNeighborhoodPhase ? rfLayoutInput : null, rfLayoutOptions);

  // Overview layout — active in overview phase only, with progressive refine.
  // Passing `null` in search/neighborhood prevents any layout computation.
  const overviewLayout = useWorkerLayout(
    isOverviewPhase ? overviewLayoutInput : null,
    overviewLayoutOptions,
    { progressive: true },
  );

  // Select the active layout based on current phase.
  // Each hook instance has isolated state — no cross-contamination possible.
  const layout = isOverviewPhase ? overviewLayout : rfLayout;

  // Record the layout duration for the P1.1 HUD (now the worker's dagre time;
  // no main-thread block).
  useEffect(() => {
    if (perfEnabled && layout.layoutMs != null) {
      perfMonitorRef.current?.recordLayout(layout.layoutMs);
    }
  }, [perfEnabled, layout.layoutMs]);

  // Apply worker positions, respecting manual positions for dragged nodes.
  // Atomic: the whole position Map lands in one commit.
  const laidOutNodes = useMemo(() => {
    const positions = layout.positions;
    if (!positions || positions.size === 0) return initialNodes;
    return initialNodes.map((node) => {
      const manual = manualPositions.get(node.id);
      if (manual) return { ...node, position: manual };
      const auto = positions.get(node.id);
      if (auto) return { ...node, position: auto };
      return node;
    });
  }, [initialNodes, layout.positions, manualPositions]);

  // Inject current LOD into node data — only recomputes when the level changes.
  // The dispatcher (ErTableNode) switches render trees on `lod`.
  // Gate 4 D2 (#42): the focused node hydrates to "detail" regardless of zoom,
  // so the seed/selected table always shows full column lists while neighbors
  // stay at the global compact/summary tier.
  const tieredNodes = useMemo(() => {
    return laidOutNodes.map((node) => {
      const d = node.data as TableNodeData;
      const targetLod = focusedNodeId && node.id === focusedNodeId ? "detail" : currentLod;
      if (d.lod === targetLod) return node;
      return { ...node, data: { ...d, lod: targetLod } };
    });
  }, [laidOutNodes, currentLod, focusedNodeId]);

  const [nodes, setNodes, onNodesChange] = useNodesState(laidOutNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

  // Sync tieredNodes to React Flow state (layout already computed in useMemo above)
  useEffect(() => {
    setNodes(tieredNodes);
  }, [tieredNodes, setNodes]);

  // Edge highlighting only — no layout re-computation
  useEffect(() => {
    if (!selectedEdgeId) {
      setEdges(initialEdges);
      return;
    }
    setEdges(
      initialEdges.map((e) => ({
        ...e,
        style:
          e.id === selectedEdgeId
            ? { strokeWidth: 2.5, stroke: "var(--accent)" }
            : { strokeWidth: 1, opacity: 0.3 },
        animated: e.id === selectedEdgeId,
      })),
    );
  }, [selectedEdgeId, initialEdges, setEdges]);

  // Fit view once a new position set commits (P1.7). Keyed on the position
  // Map identity, so it fires for both the worker path (computing → ready) and
  // the cache-hit path (idle → ready, which never passes through computing),
  // exactly once per distinct layout. Replaces the old fixed-80ms timer, which
  // could fire before positions landed. The short delay lets React Flow
  // measure the freshly-mounted nodes before fitting (F-MR-1: uses the
  // captured instance's fitView, not a synthetic keydown).
  useEffect(() => {
    if (layout.status !== "ready" || !layout.positions) return;
    if (fittedPositionsRef.current === layout.positions) return;
    fittedPositionsRef.current = layout.positions;
    const timer = setTimeout(() => {
      rfInstanceRef.current?.fitView({ padding: 0.2 });
    }, 60);
    return () => clearTimeout(timer);
  }, [layout.status, layout.positions]);

  // Filter nodes by search (small schemas only — large schemas use neighborhood)
  useEffect(() => {
    if (isLargeSchema) return;
    if (!searchQuery.trim()) {
      setNodes(tieredNodes);
      return;
    }
    const q = searchQuery.toLowerCase();
    setNodes(
      tieredNodes.map((n) => ({
        ...n,
        style: {
          ...((n.data as TableNodeData).label.toLowerCase().includes(q) ? {} : { opacity: 0.3 }),
        },
      })),
    );
  }, [searchQuery, tieredNodes, setNodes, isLargeSchema]);

  // Capture the mounted instance (F-MR-1) and fit view on first render.
  const onInit = useCallback(
    (instance: ReactFlowInstance) => {
      rfInstanceRef.current = instance;
      // Small delay to ensure nodes are rendered
      setTimeout(() => instance.fitView({ padding: 0.2 }), 100);
      // React Flow is mounted and interactive → that is the "interactive shell".
      if (perfEnabled) {
        perfMonitorRef.current?.recordInit(performance.now() - renderStartRef.current);
      }
    },
    [perfEnabled],
  );

  // Sample rAF frame times during pan/zoom gestures.
  const onMoveStart = useCallback(() => {
    if (perfEnabled) perfMonitorRef.current?.beginFrameSampling();
  }, [perfEnabled]);

  const onMoveEnd = useCallback(() => {
    if (perfEnabled) perfMonitorRef.current?.endFrameSampling();
  }, [perfEnabled]);

  // Open the table's detail tab — the EXPLICIT action on the overview
  // (PR#12 re-review P1): double-click a node or the side inspector's "Open
  // table" button. A single click focuses the neighborhood instead and must
  // never navigate away from the diagram.
  const openTableObject = useCallback(
    (schemaName: string, tableName: string) => {
      useWorkspaceStore.getState().openDbObject({
        id: `dbobj:${schemaName}.${tableName}:${connectionId}`,
        kind: "db-object",
        title: tableName,
        connectionId,
        resourceKey: `dbobj:${schemaName}.${tableName}:${connectionId}`,
        dirty: false,
        pinned: false,
        preview: false,
        order: Date.now(),
        data: {
          schema: schemaName,
          objectName: tableName,
          objectType: "table",
          activeSection: "columns",
        },
      });
    },
    [connectionId],
  );

  // Click node behavior depends on the phase:
  // - Neighborhood: single click focuses the node (hydrates to detail LOD).
  // - Otherwise: click opens the table (legacy small/medium schema behavior).
  const onNodeClick = useCallback(
    (_: React.MouseEvent, node: Node) => {
      const nodeData = node.data as TableNodeData;
      if (isNeighborhoodPhase) {
        dispatchLargeSchema({ type: "FOCUS_NODE", nodeKey: node.id });
        return;
      }
      openTableObject(nodeData.schema, nodeData.label);
    },
    [openTableObject, isNeighborhoodPhase],
  );

  // Explicit open-table action for the canvas overview: fed to the view's
  // `onOpenTable`, fired on double-click / the side inspector button.
  const onCytoscapeNodeClick = useCallback(
    (nodeId: string) => {
      const table = graphModel.tables.find((t) => t.id === nodeId);
      if (table) openTableObject(table.schema, table.label);
    },
    [graphModel, openTableObject],
  );

  const onCytoscapeViewportChange = useCallback((viewport: ErViewport) => {
    viewportRef.current = { x: viewport.x, y: viewport.y, zoom: viewport.zoom };
  }, []);

  // Click edge → highlight it (only meaningful at full edge LOD, where edge
  // ids are stable; aggregated/simple edges are transient render artifacts).
  const onEdgeClick = useCallback(
    (_: React.MouseEvent, edge: Edge) => {
      if (currentEdgeLod !== "full") return;
      setSelectedEdgeId((prev) => (prev === edge.id ? null : edge.id));
    },
    [currentEdgeLod],
  );

  // Clear edge selection when leaving full edge LOD — aggregated edge ids are
  // synthetic and must not leak into the highlight effect over initialEdges.
  useEffect(() => {
    if (currentEdgeLod !== "full") setSelectedEdgeId(null);
  }, [currentEdgeLod]);

  // Click background → clear edge selection + clear focus (neighborhood)
  const onPaneClick = useCallback(() => {
    setSelectedEdgeId(null);
    if (isNeighborhoodPhase) {
      dispatchLargeSchema({ type: "CLEAR_FOCUS" });
    }
  }, [isNeighborhoodPhase]);

  // Listen for column click events from TableNode → navigate to Columns section
  useEffect(() => {
    const handler = (e: Event) => {
      const detail = (e as CustomEvent).detail as { tableName: string; columnName: string };
      const table = data.tables.find((t) => t.name === detail.tableName && t.schema === schema);
      if (!table) return;
      useWorkspaceStore.getState().openDbObject({
        id: `dbobj:${table.schema}.${table.name}:${connectionId}`,
        kind: "db-object",
        title: table.name,
        connectionId,
        resourceKey: `dbobj:${table.schema}.${table.name}:${connectionId}`,
        dirty: false,
        pinned: false,
        preview: false,
        order: Date.now(),
        data: {
          schema: table.schema,
          objectName: table.name,
          objectType: "table",
          activeSection: "columns",
        },
      });
    };
    document.addEventListener("er-column-click", handler);
    return () => document.removeEventListener("er-column-click", handler);
  }, [connectionId, data.tables, schema]);

  // Persist node position after drag
  const onNodeDragStop = useCallback(
    (_: MouseEvent | TouchEvent, node: Node) => {
      setManualPositions((prev) => {
        const next = new Map(prev);
        next.set(node.id, node.position);
        savePositions(next);
        return next;
      });
    },
    [savePositions],
  );

  // Reset all manual positions
  const handleResetLayout = useCallback(() => {
    setManualPositions(new Map());
    savePositions(new Map());
  }, [savePositions]);

  const handleFitView = useCallback(() => {
    rfInstanceRef.current?.fitView({ padding: 0.2 });
  }, []);

  // #44 E1 — explicit neighborhood → overview transition.
  // The ONLY path from bounded neighborhood to full overview is this action.
  // No zoom/pan/search/layout completion may trigger the transition.
  const handleShowAll = useCallback(() => {
    dispatchLargeSchema({ type: "SHOW_ALL" });
  }, []);

  // #46 E3 — explicit back navigation from overview to neighborhood.
  // Preserves seedTable (the reducer keeps it), clears focusedNodeId.
  // The useLayoutEffect phase-transition handler clears fittedPositionsRef
  // and resets LOD to "compact" — no stale overview state flows to React Flow.
  const handleBackToNeighborhood = useCallback(() => {
    dispatchLargeSchema({ type: "BACK_TO_NEIGHBORHOOD" });
  }, []);

  // #46 E3 — explicit back navigation from overview to search.
  // Resets to initialLargeSchemaState: phase=search, seedTable=null, focusedNodeId=null.
  const handleBackToSearch = useCallback(() => {
    dispatchLargeSchema({ type: "BACK_TO_SEARCH" });
  }, []);

  // P1.5 spatial index over the live React Flow node/edge state.
  //
  // Rebuild-on-identity-change is deliberate: `nodes` is replaced by React
  // Flow on measurement updates, drag frames, and LOD-threshold crossings, and
  // rebuilding keeps the index faithful to measured sizes + drag positions.
  // Each rebuild is O(N) with cheap map ops — do not memoize this into a stale
  // snapshot. Viewport queries stay O(cells + overlaps). Consumed by the HUD;
  // later by the Viewport Engine (P1.6) and a future Canvas renderer.
  const spatialIndex = useMemo(() => {
    const index = new SpatialIndex();
    index.build(
      nodes.map((n) => ({
        id: n.id,
        position: n.position,
        measured: n.measured,
      })),
      edges.map((e) => ({ id: e.id, source: e.source, target: e.target })),
    );
    return index;
  }, [nodes, edges]);

  // Edge LOD (P1.4) + node-LOD handle interplay (P1.3).
  //
  // Per-column handle ids only exist on detail nodes. React Flow drops edges
  // whose handle ids are missing (error 008), so whenever nodes are below
  // detail we strip handle ids and let edges anchor to the generic
  // source/target handles rendered by the non-detail LOD leaves (`handles[0]`
  // fallback — valid only because generic handles are the only handles there).
  //
  // Edge bands (locked): aggregate < 0.25 (merged relations, straight, no
  // markers/labels except count) · simple 0.25–0.6 (straight, no markers) ·
  // full > 0.6 (normal FK edges).
  const displayEdges = useMemo(() => {
    const stripHandleIds = currentLod !== "detail";
    const stripHandles = (e: Edge) =>
      stripHandleIds ? { ...e, sourceHandle: undefined, targetHandle: undefined } : e;
    if (currentEdgeLod === "aggregate") {
      // Node LOD never reaches detail below zoom 0.7, so stripHandleIds is
      // always true here; stripHandles is applied for consistency anyway.
      return aggregateRelations(edges).map((rel) =>
        stripHandles({
          id: `fk:agg:${rel.source}:${rel.target}`,
          source: rel.source,
          target: rel.target,
          type: "straight",
          animated: false,
          label: rel.count > 1 ? String(rel.count) : undefined,
          labelStyle: { fontSize: 9, opacity: 0.6 },
          markerEnd: undefined,
          style: { strokeWidth: 1 },
        }),
      );
    }

    if (currentEdgeLod === "simple") {
      return edges.map((e) =>
        stripHandles({
          ...e,
          type: "straight",
          label: undefined,
          markerEnd: undefined,
          style: { ...e.style, strokeWidth: 1 },
        }),
      );
    }

    // Full edge LOD: keep markers/labels/paths. Still strip handle ids while
    // nodes are below detail (0.6–0.7 band) so edges stay anchored.
    if (stripHandleIds) {
      return edges.map((e) => stripHandles({ ...e, style: { ...e.style, strokeWidth: 1 } }));
    }
    return edges;
  }, [edges, currentLod, currentEdgeLod]);

  // Highlight hop radius for the canvas overview (opass-style neighborhood
  // #46 E3 — overview highlight hops changes ONLY the canvas highlight ring.
  // Must NOT mutate neighborhoodHops (which is preserved for Back navigation).
  const handleHighlightHops = useCallback((hops: NeighborhoodScope) => {
    setOverviewHighlightHops(hops);
  }, []);

  const handleSelectTable = useCallback((tableKey: string) => {
    dispatchLargeSchema({ type: "SELECT_TABLE", tableKey });
  }, []);

  const showMiniMap = initialNodes.length <= 200;

  return (
    <div className="flex min-h-0 flex-1 flex-col" ref={reactFlowRef}>
      {isSearchPhase ? (
        <ErSearchEntry model={graphModel} onSelectTable={handleSelectTable} />
      ) : activeCytoscape ? (
        <Suspense
          fallback={
            <div className="flex min-h-0 flex-1 items-center justify-center gap-2 text-[12px] text-[var(--text-secondary)]">
              <Loader2 className="h-3.5 w-3.5 animate-spin" />
              Loading overview…
            </div>
          }
        >
          <CytoscapeErView
            model={graphModel}
            positions={layout.positions}
            degraded={layout.degraded}
            layoutReady={layout.status === "ready"}
            onViewportChange={onCytoscapeViewportChange}
            onOpenTable={onCytoscapeNodeClick}
            explorer={{
              totalTables: tablesInSchema.length,
              relationCount: schemaStats.relations,
              columnCount: schemaStats.columns,
              hops: overviewHighlightHops,
              onSelectHops: handleHighlightHops,
            }}
            searchQuery={searchQuery}
            onSearchChange={setSearchQuery}
            onBackToNeighborhood={handleBackToNeighborhood}
            onBackToSearch={handleBackToSearch}
          />
        </Suspense>
      ) : (
        <ReactFlow
          nodes={nodes}
          edges={displayEdges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onViewportChange={onViewportChange}
          onMoveStart={onMoveStart}
          onMoveEnd={onMoveEnd}
          onInit={onInit}
          onNodeClick={onNodeClick}
          onEdgeClick={onEdgeClick}
          onPaneClick={onPaneClick}
          onNodeDragStop={onNodeDragStop}
          nodeTypes={nodeTypes}
          fitView
          fitViewOptions={{ padding: 0.2 }}
          onlyRenderVisibleElements
          minZoom={0.1}
          maxZoom={2}
          proOptions={{ hideAttribution: true }}
          nodesDraggable
          nodesConnectable={false}
          deleteKeyCode={null}
          className="bg-background"
        >
          <Background
            variant={BackgroundVariant.Dots}
            gap={20}
            size={1}
            color="var(--border-subtle)"
          />
          <Controls showInteractive={false} />

          {/* P1.7 — layout is computing in the Worker; shell stays interactive. */}
          {layout.status === "computing" && (
            <Panel position="top-center" className="m-2">
              <div className="flex items-center gap-2 rounded-md border bg-popover px-3 py-1.5 text-[12px] text-muted-foreground shadow-sm">
                <Loader2 className="h-3.5 w-3.5 animate-spin" />
                Arranging {layout.nodeCount} tables…
              </div>
            </Panel>
          )}
          {layout.status === "error" && (
            <Panel position="top-center" className="m-2">
              <div className="flex items-center gap-2 rounded-md border border-destructive/40 bg-destructive/10 px-3 py-1.5 text-[12px] text-destructive">
                Layout failed: {layout.error}
              </div>
            </Panel>
          )}

          {perfEnabled && perfMonitorRef.current && (
            <ErPerfHud
              monitor={perfMonitorRef.current}
              spatialIndex={spatialIndex}
              edgeCount={edges.length}
              viewportRef={viewportRef}
            />
          )}
          {showMiniMap && (
            <MiniMap
              nodeColor={(n) => {
                const d = n.data as TableNodeData;
                return d.columns?.some((c) => c.isForeignKey)
                  ? "var(--state-info)"
                  : "var(--accent)";
              }}
              maskColor="rgba(0,0,0,0.08)"
              className="!bg-popover !border-[var(--border-default)]"
            />
          )}

          {/* Top-left panel: search + P1.6 neighborhood exploration */}
          <Panel position="top-left" className="m-2">
            <div className="flex flex-col items-start gap-2">
              <div className="flex items-center gap-2">
                <div className="relative">
                  <Search className="absolute left-2 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-[var(--text-secondary)]" />
                  <Input
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    placeholder={`${t("shell.sidebar.searchObjects")}...`}
                    className="h-7 w-48 rounded-md pl-7 text-[12px]"
                  />
                </div>
                {!isLargeSchema && (
                  <Badge variant="outline" className="h-7 text-[11px]">
                    <Table2 className="mr-1 h-3 w-3" />
                    {initialNodes.length} tables
                  </Badge>
                )}
              </div>
            </div>
          </Panel>

          {/* Top-right: #44 E1 — explicit Show All transition (neighborhood only) */}
          {isNeighborhoodPhase && (
            <Panel position="top-right" className="m-2">
              <Button
                type="button"
                variant="outline"
                size="sm"
                data-testid="er-show-all-tables"
                aria-label={`Show all ${tablesInSchema.length} tables`}
                onClick={handleShowAll}
              >
                Show all {tablesInSchema.length} tables
              </Button>
            </Panel>
          )}

          {/* Bottom-left: layout controls */}
          <Panel position="bottom-left" className="m-2">
            <div className="flex items-center gap-1">
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="h-7 w-7 p-0"
                    onClick={() => setLayoutDirection((d) => (d === "LR" ? "TB" : "LR"))}
                  >
                    <LayoutGrid className="h-3.5 w-3.5" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Toggle layout direction</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="h-7 w-7 p-0"
                    onClick={() => setCompact((c) => !c)}
                  >
                    <Columns2 className="h-3.5 w-3.5" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>{compact ? "Show columns" : "Compact mode"}</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="h-7 w-7 p-0"
                    onClick={handleFitView}
                  >
                    <Maximize2 className="h-3.5 w-3.5" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Fit view</TooltipContent>
              </Tooltip>
              {manualPositions.size > 0 && (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      className="h-7 w-7 p-0"
                      onClick={handleResetLayout}
                    >
                      <RotateCcw className="h-3.5 w-3.5" />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>Reset layout</TooltipContent>
                </Tooltip>
              )}
            </div>
          </Panel>
        </ReactFlow>
      )}
    </div>
  );
}
