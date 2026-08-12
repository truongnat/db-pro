import { useCallback, useEffect, useMemo, useRef, useState } from "react";
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
import {
  Search,
  Maximize2,
  LayoutGrid,
  Columns2,
  Table2,
  RotateCcw,
  Eye,
  Focus,
} from "lucide-react";

import { ErTableNode, type TableNodeData } from "./lod/er-table-node";
import { ErPerfHud } from "./er-perf-hud";
import { layoutGraph } from "../utils/layout";
import { groupForeignKeys } from "../utils/edge-builder";
import { buildAdjacencyMap, getNeighborhood } from "../utils/neighborhood";
import { ErPerfMonitor } from "../utils/instrumentation";
import { resolveLod, type LodLevel } from "../utils/lod";
import { aggregateRelations, resolveEdgeLod, type EdgeLodLevel } from "../utils/edge-lod";

import type { IntrospectResult, SchemaColumnDto } from "@/modules/schema/types/schema.types";

const LARGE_SCHEMA_THRESHOLD = 200;
const NEIGHBORHOOD_HOPS = 2;

interface ErDiagramProps {
  connectionId: string;
  schema: string;
  data: IntrospectResult;
}

const nodeTypes = { table: ErTableNode };

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

  // Neighborhood mode for large schemas
  const tablesInSchema = data.tables.filter((t) => t.schema === schema);
  const isLargeSchema = tablesInSchema.length > LARGE_SCHEMA_THRESHOLD;
  const [neighborhoodSeed, setNeighborhoodSeed] = useState<string | null>(null);
  const [showAll, setShowAll] = useState(false);

  const adjacencyMap = useMemo(() => buildAdjacencyMap(data.foreignKeys), [data.foreignKeys]);

  const neighborhoodSet = useMemo(() => {
    if (!isLargeSchema || showAll || !neighborhoodSeed) return null;
    return getNeighborhood(adjacencyMap, neighborhoodSeed, NEIGHBORHOOD_HOPS);
  }, [isLargeSchema, showAll, neighborhoodSeed, adjacencyMap]);

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

  // Pre-index metadata by schema.tableName — O(C + P + F) once, O(1) per table lookup
  const columnsByTable = useMemo(() => {
    const map = new Map<string, SchemaColumnDto[]>();
    for (const col of data.columns) {
      const key = `${col.schema}.${col.tableName}`;
      const list = map.get(key);
      if (list) list.push(col);
      else map.set(key, [col]);
    }
    return map;
  }, [data.columns]);

  const primaryKeysByTable = useMemo(() => {
    const map = new Map<string, Set<string>>();
    for (const pk of data.primaryKeys) {
      const key = `${pk.schema}.${pk.tableName}`;
      const existing = map.get(key);
      if (existing) {
        for (const c of pk.columns) existing.add(c);
      } else {
        map.set(key, new Set(pk.columns));
      }
    }
    return map;
  }, [data.primaryKeys]);

  const fkColumnSet = useMemo(() => {
    const set = new Set<string>();
    for (const fk of data.foreignKeys) {
      set.add(`${fk.schema}.${fk.fromTable}:${fk.fromColumn}`);
    }
    return set;
  }, [data.foreignKeys]);

  // Build nodes and edges from introspection data using pre-indexed maps
  const { initialNodes, initialEdges } = useMemo(() => {
    const tables = neighborhoodSet
      ? data.tables.filter(
          (t) => t.schema === schema && neighborhoodSet.has(`${t.schema}.${t.name}`),
        )
      : data.tables.filter((tbl) => tbl.schema === schema);

    const nodes: Node[] = tables.map((table) => {
      const tableKey = `${table.schema}.${table.name}`;
      const cols = columnsByTable.get(tableKey);
      const pkCols = primaryKeysByTable.get(tableKey);

      const columnData = (cols ?? []).map((col) => ({
        name: col.name,
        dataType: col.dataType,
        nullable: col.nullable,
        isPrimaryKey: pkCols?.has(col.name) ?? false,
        isForeignKey: fkColumnSet.has(`${tableKey}:${col.name}`),
      }));

      const nodeData: TableNodeData = {
        label: table.name,
        schema: table.schema,
        columns: columnData,
        compact,
        // Initial value; the lod-injection memo below refines it per viewport.
        lod: "detail",
      };

      return {
        id: tableKey,
        type: "table",
        position: { x: 0, y: 0 },
        data: nodeData,
      };
    });

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
  }, [data, compact, schema, columnsByTable, primaryKeysByTable, fkColumnSet, neighborhoodSet]);

  // Apply layout, respecting manual positions for dragged nodes
  const laidOutNodes = useMemo(() => {
    const layoutStart = performance.now();
    const autoLaid = layoutGraph(initialNodes, initialEdges, { direction: layoutDirection });
    if (perfEnabled) perfMonitorRef.current?.recordLayout(performance.now() - layoutStart);
    return autoLaid.map((node) => {
      const manual = manualPositions.get(node.id);
      if (manual) return { ...node, position: manual };
      return node;
    });
  }, [initialNodes, initialEdges, layoutDirection, manualPositions, perfEnabled]);

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
            ? { strokeWidth: 2.5, stroke: "var(--primary)" }
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

  // Click node → open table tab
  const onNodeClick = useCallback(
    (_: React.MouseEvent, node: Node) => {
      const nodeData = node.data as TableNodeData;
      useWorkspaceStore.getState().openDbObject({
        id: `dbobj:${nodeData.schema}.${nodeData.label}:${connectionId}`,
        kind: "db-object",
        title: nodeData.label,
        connectionId,
        resourceKey: `dbobj:${nodeData.schema}.${nodeData.label}:${connectionId}`,
        dirty: false,
        pinned: false,
        preview: false,
        order: Date.now(),
        data: {
          schema: nodeData.schema,
          objectName: nodeData.label,
          objectType: "table",
          activeSection: "columns",
        },
      });
    },
    [connectionId],
  );

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

  const showMiniMap = initialNodes.length <= 200;

  return (
    <div className="flex min-h-0 flex-1 flex-col" ref={reactFlowRef}>
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
          color="var(--app-border-subtle)"
        />
        <Controls showInteractive={false} />
        {perfEnabled && perfMonitorRef.current && (
          <ErPerfHud
            monitor={perfMonitorRef.current}
            nodes={nodes}
            edgeCount={edges.length}
            viewportRef={viewportRef}
          />
        )}
        {showMiniMap && (
          <MiniMap
            nodeColor={(n) => {
              const d = n.data as TableNodeData;
              return d.columns?.some((c) => c.isForeignKey) ? "var(--info)" : "var(--primary)";
            }}
            maskColor="rgba(0,0,0,0.08)"
            className="!bg-popover !border-[var(--app-border)]"
          />
        )}

        {/* Top panel: search + controls */}
        <Panel position="top-left" className="m-2">
          <div className="flex items-center gap-2">
            <div className="relative">
              <Search className="absolute left-2 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-[var(--app-text-muted)]" />
              <Input
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder={`${t("shell.sidebar.searchObjects")}...`}
                className="h-7 w-48 rounded-md pl-7 text-[12px]"
              />
            </div>
            <Badge variant="outline" className="h-7 text-[11px]">
              <Table2 className="mr-1 h-3 w-3" />
              {initialNodes.length}
              {neighborhoodSet ? ` / ${tablesInSchema.length}` : ""} tables
            </Badge>
            {isLargeSchema && neighborhoodSet && (
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="h-7 text-[11px]"
                    onClick={() => {
                      setShowAll(true);
                      setNeighborhoodSeed(null);
                    }}
                  >
                    <Eye className="mr-1 h-3 w-3" />
                    Show all {tablesInSchema.length}
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Render all tables in schema</TooltipContent>
              </Tooltip>
            )}
            {isLargeSchema && showAll && (
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="h-7 text-[11px]"
                    onClick={() => {
                      setShowAll(false);
                      setNeighborhoodSeed(null);
                      setSearchQuery("");
                    }}
                  >
                    <Focus className="mr-1 h-3 w-3" />
                    Neighborhood mode
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Search a table to focus its neighborhood</TooltipContent>
              </Tooltip>
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
    </div>
  );
}
