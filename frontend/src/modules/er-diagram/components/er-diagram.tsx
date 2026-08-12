import { lazy, Suspense, useCallback, useEffect, useMemo, useRef, useState } from "react";
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
import { NeighborhoodExplorer } from "./neighborhood-explorer";
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
import { groupForeignKeys } from "../utils/edge-builder";
import {
  buildAdjacencyMap,
  getConnectedComponent,
  getNeighborhood,
  suggestStartingPoints,
  type NeighborhoodScope,
} from "../utils/neighborhood";
import { ErPerfMonitor } from "../utils/instrumentation";
import { resolveLod, type LodLevel } from "../utils/lod";
import { aggregateRelations, resolveEdgeLod, type EdgeLodLevel } from "../utils/edge-lod";
import { SpatialIndex } from "../utils/spatial-index";

import type { IntrospectResult } from "@/modules/schema/types/schema.types";
import { buildErNodeIndexes, buildTableNodes } from "../renderer/er-node-builder";

const SUGGESTED_POINTS_COUNT = 5;

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

export function ErDiagram({ connectionId, schema, data }: ErDiagramProps) {
  const { t } = useTranslation();
  const reactFlowRef = useRef<HTMLDivElement>(null);

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

  // Neighborhood mode for large schemas (P1.6 exploration UX)
  const tablesInSchema = data.tables.filter((t) => t.schema === schema);

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

  const [neighborhoodSeed, setNeighborhoodSeed] = useState<string | null>(null);
  const [showAll, setShowAll] = useState(false);
  const [neighborhoodHops, setNeighborhoodHops] = useState<NeighborhoodScope>(2);

  const adjacencyMap = useMemo(() => buildAdjacencyMap(data.foreignKeys), [data.foreignKeys]);

  // P1.9 — the explicit "All N tables" overview of a large schema renders on
  // the canvas renderer (CytoscapeErRenderer). Landing and neighborhood modes
  // keep React Flow (small node sets where its interaction richness wins).
  const useCytoscapeForOverview = isLargeSchema && showAll;

  // Landing = large schema, no seed yet, not showing everything: exploration
  // mode, empty canvas (full schema is not the default experience).
  const landing = isLargeSchema && !neighborhoodSeed && !showAll;

  const neighborhoodSet = useMemo(() => {
    if (!isLargeSchema || showAll || !neighborhoodSeed) return null;
    if (neighborhoodHops === "domain") {
      return getConnectedComponent(adjacencyMap, neighborhoodSeed);
    }
    return getNeighborhood(adjacencyMap, neighborhoodSeed, neighborhoodHops);
  }, [isLargeSchema, showAll, neighborhoodSeed, neighborhoodHops, adjacencyMap]);

  // Hub tables by FK degree — the "Suggested starting points" list.
  const suggestedPoints = useMemo(() => {
    if (!isLargeSchema) return [];
    return suggestStartingPoints(adjacencyMap, tablesInSchema, SUGGESTED_POINTS_COUNT).map(
      (key) => ({ key, label: key.slice(key.indexOf(".") + 1) }),
    );
  }, [isLargeSchema, adjacencyMap, tablesInSchema]);

  // Schema statistics for the landing card.
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

  // Persisted manual positions: nodeId → { x, y }
  const [manualPositions, setManualPositions] = useState<Map<string, { x: number; y: number }>>(
    () => {
      try {
        const raw = localStorage.getItem(positionStorageKey(connectionId, schema));
        if (raw) return new Map(JSON.parse(raw));
      } catch {
        /* ignore */
      }
      return new Map();
    },
  );

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
  // Landing mode renders an empty canvas (exploration panel only).
  const { initialNodes, initialEdges } = useMemo(() => {
    const tables = landing
      ? []
      : neighborhoodSet
        ? data.tables.filter(
            (t) => t.schema === schema && neighborhoodSet.has(`${t.schema}.${t.name}`),
          )
        : data.tables.filter((tbl) => tbl.schema === schema);

    // Pure pre-indexed build (P3.3) — O(1) map lookups, no per-table scans
    // of data.columns / data.primaryKeys / data.foreignKeys.
    const nodes: Node[] = buildTableNodes(tables, nodeIndexes, { compact });

    const visibleTableKeys = new Set(tables.map((t) => `${t.schema}.${t.name}`));
    const fkGroups = groupForeignKeys(data.foreignKeys, visibleTableKeys);

    const edges: Edge[] = fkGroups.map((group) => ({
      id: `fk:${group.key}`,
      source: `${group.fk.toSchema}.${group.fk.toTable}`,
      target: `${group.fk.schema}.${group.fk.fromTable}`,
      sourceHandle: `pk:${group.fk.toColumn}`,
      targetHandle: `fk:${group.fk.fromColumn}`,
      type: "smoothstep",
      animated: false,
      markerEnd: { type: MarkerType.ArrowClosed, width: 12, height: 12 },
      style: { strokeWidth: 1.5 },
      label: group.fk.name,
      labelStyle: { fontSize: 9, opacity: 0.6 },
    }));

    return { initialNodes: nodes, initialEdges: edges };
  }, [data, compact, schema, nodeIndexes, neighborhoodSet, landing]);

  // P1.7 — layout off the main thread.
  //
  // Build a plain (worker-serializable) layout input from the RF nodes, run
  // dagre in the Worker (cache-first by content hash), and commit positions
  // atomically when the result lands. The UI stays interactive meanwhile and
  // shows an "Arranging N tables…" overlay (locked: no fake progressive
  // layout — positions only appear once stable).
  const layoutInput = useMemo<LayoutInput | null>(() => {
    if (initialNodes.length === 0) return null;
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
  }, [initialNodes, initialEdges]);

  // Stable identity — `useWorkerLayout` re-runs layout only when the hash
  // (content + options) changes, not on every render.
  const layoutOptions = useMemo(() => ({ direction: layoutDirection }), [layoutDirection]);

  const layout = useWorkerLayout(layoutInput, layoutOptions);

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
  const tieredNodes = useMemo(() => {
    return laidOutNodes.map((node) => {
      const d = node.data as TableNodeData;
      if (d.lod === currentLod) return node;
      return { ...node, data: { ...d, lod: currentLod } };
    });
  }, [laidOutNodes, currentLod]);

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

  // In large schemas, search triggers neighborhood mode
  useEffect(() => {
    if (!isLargeSchema) return;
    if (!searchQuery.trim()) {
      if (!showAll) setNeighborhoodSeed(null);
      return;
    }
    const q = searchQuery.toLowerCase();
    const match = tablesInSchema.find((t) => t.name.toLowerCase().includes(q));
    if (match) {
      setNeighborhoodSeed(`${match.schema}.${match.name}`);
      setShowAll(false);
    }
  }, [searchQuery, isLargeSchema, tablesInSchema, showAll]);

  // Fit view once a new position set commits (P1.7). Keyed on the position
  // Map identity, so it fires for both the worker path (computing → ready) and
  // the cache-hit path (idle → ready, which never passes through computing),
  // exactly once per distinct layout. Replaces the old fixed-80ms timer, which
  // could fire before positions landed.
  const fittedPositionsRef = useRef<Map<string, LayoutPosition> | null>(null);
  useEffect(() => {
    if (layout.status !== "ready" || !layout.positions) return;
    if (fittedPositionsRef.current === layout.positions) return;
    fittedPositionsRef.current = layout.positions;
    const timer = setTimeout(() => {
      const rfNode = reactFlowRef.current?.querySelector(".react-flow") as HTMLElement | null;
      rfNode?.dispatchEvent(new KeyboardEvent("keydown", { key: "1" }));
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

  // Fit view on first render
  const onInit = useCallback(
    (instance: { fitView: (opts?: Record<string, unknown>) => void }) => {
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

  // Open the table's detail tab — shared by the React Flow and Cytoscape views.
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

  // Click node → open table tab (React Flow view).
  const onNodeClick = useCallback(
    (_: React.MouseEvent, node: Node) => {
      const nodeData = node.data as TableNodeData;
      openTableObject(nodeData.schema, nodeData.label);
    },
    [openTableObject],
  );

  // Click node → open table tab (Cytoscape overview view).
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

  // Click background → clear edge selection
  const onPaneClick = useCallback(() => {
    setSelectedEdgeId(null);
  }, []);

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
    const rfNode = reactFlowRef.current?.querySelector(".react-flow") as HTMLElement | null;
    rfNode?.dispatchEvent(new KeyboardEvent("keydown", { key: "1" }));
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

  // Exploration handlers shared by both renderer views (P1.6 + P1.9).
  const handleSelectPoint = useCallback((key: string) => {
    // Do NOT clear searchQuery here: clearing it re-runs the search effect,
    // which would null the seed we just set.
    setNeighborhoodSeed(key);
    setShowAll(false);
  }, []);
  const handleSelectHops = useCallback(
    (hops: NeighborhoodScope) => {
      // Landing: picking a hop scope auto-focuses the top hub.
      if (landing && suggestedPoints[0]) setNeighborhoodSeed(suggestedPoints[0].key);
      setNeighborhoodHops(hops);
      setShowAll(false);
    },
    [landing, suggestedPoints],
  );
  const handleShowAll = useCallback(() => {
    // Clear leftover search text so the search effect cannot match a stale
    // term and flip showAll back off.
    setSearchQuery("");
    setShowAll(true);
    setNeighborhoodSeed(null);
  }, []);
  const handleResetExploration = useCallback(() => {
    setShowAll(false);
    setNeighborhoodSeed(null);
    setSearchQuery("");
    setNeighborhoodHops(2);
  }, []);

  const showMiniMap = !landing && initialNodes.length <= 200;

  return (
    <div className="flex min-h-0 flex-1 flex-col" ref={reactFlowRef}>
      {useCytoscapeForOverview ? (
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
            positions={layout.status === "ready" ? layout.positions : null}
            layoutStatus={layout.status}
            onViewportChange={onCytoscapeViewportChange}
            onNodeClick={onCytoscapeNodeClick}
            explorer={{
              totalTables: tablesInSchema.length,
              relationCount: schemaStats.relations,
              columnCount: schemaStats.columns,
              suggestedPoints,
              seedLabel: null,
              hops: neighborhoodHops,
              showAll: true,
              onSelectPoint: handleSelectPoint,
              onSelectHops: handleSelectHops,
              onShowAll: handleShowAll,
              onReset: handleResetExploration,
            }}
            searchQuery={searchQuery}
            onSearchChange={setSearchQuery}
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
              {isLargeSchema && (
                <NeighborhoodExplorer
                  totalTables={tablesInSchema.length}
                  relationCount={schemaStats.relations}
                  columnCount={schemaStats.columns}
                  suggestedPoints={suggestedPoints}
                  seedLabel={
                    neighborhoodSeed
                      ? neighborhoodSeed.slice(neighborhoodSeed.indexOf(".") + 1)
                      : null
                  }
                  hops={neighborhoodHops}
                  showAll={showAll}
                  onSelectPoint={handleSelectPoint}
                  onSelectHops={handleSelectHops}
                  onShowAll={handleShowAll}
                  onReset={handleResetExploration}
                />
              )}
            </div>
          </Panel>

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
