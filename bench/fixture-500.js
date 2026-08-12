/*
 * ER benchmark fixture — shared by both harnesses (Cytoscape vs React Flow).
 *
 * Deterministic (mulberry32 seeded PRNG): every load produces identical data, so
 * harness A and harness B benchmark the exact same graph.
 *
 * Shape: 500 tables across 20 domains, ~900 FK relations (65% intra-domain,
 * 35% cross-domain), ~7,500 columns. Exposed as `window.ER_FIXTURE`.
 */
(function () {
  "use strict";

  function mulberry32(seed) {
    let a = seed >>> 0;
    return function () {
      a |= 0;
      a = (a + 0x6d2b79f5) | 0;
      let t = Math.imul(a ^ (a >>> 15), 1 | a);
      t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
      return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    };
  }

  const rand = mulberry32(42);
  const pick = (arr) => arr[Math.floor(rand() * arr.length)];
  const randInt = (min, max) => min + Math.floor(rand() * (max - min + 1));

  const DOMAINS = [
    "customers", "orders", "products", "payments", "accounts",
    "shipping", "inventory", "billing", "hr", "crm",
    "marketing", "support", "analytics", "content", "subscriptions",
    "warehouse", "finance", "logistics", "catalog", "reports",
  ];
  const TABLE_SUFFIX = ["", "", "", "_history", "_archive", "_audit", "_log", "_draft"];
  const COLUMN_TYPES = [
    "bigint", "integer", "varchar(255)", "text", "timestamp",
    "timestamptz", "boolean", "numeric(12,2)", "uuid", "date", "jsonb", "double precision",
  ];
  const COMMON_COLS = ["id", "created_at", "updated_at", "status", "is_active", "version", "tenant_id", "notes", "metadata"];

  const tables = [];
  const relations = [];
  const tableById = new Map();
  let tableSeq = 1;

  // 20 domains × 25 tables = 500 tables, hub table first (domain name).
  for (const domain of DOMAINS) {
    for (let i = 0; i < 25; i++) {
      const id = "t" + String(tableSeq++).padStart(4, "0");
      const suffix = pick(TABLE_SUFFIX);
      const name = i === 0 ? domain : domain + "_" + i + suffix;
      const nCols = randInt(10, 20); // mean 15 → ~7,500 columns
      const columns = [];
      for (let c = 0; c < nCols; c++) {
        const isPk = c === 0;
        columns.push({
          name: isPk ? "id" : COMMON_COLS[c % COMMON_COLS.length] + (c < 4 ? "" : "_" + c),
          type: isPk ? "bigint" : pick(COLUMN_TYPES),
          isPk,
          isFk: false,
        });
      }
      const table = { id, name, domain, columns };
      tables.push(table);
      tableById.set(id, table);
    }
  }

  // ~900 relations. Each relation = one FK column appended to the source table.
  const targetRelations = 900;
  const seenPairs = new Set();
  let guard = 0;
  while (relations.length < targetRelations && guard++ < 40000) {
    const source = pick(tables);
    let target;
    if (rand() < 0.65) {
      const sameDomain = tables.filter((t) => t.domain === source.domain && t.id !== source.id);
      target = sameDomain[Math.floor(rand() * sameDomain.length)];
    } else {
      const others = tables.filter((t) => t.domain !== source.domain);
      target = others[Math.floor(rand() * others.length)];
    }
    if (!target) continue;
    const key = source.id + "->" + target.id;
    if (seenPairs.has(key)) continue;
    seenPairs.add(key);
    source.columns.push({ name: target.name + "_id", type: "bigint", isPk: false, isFk: true });
    relations.push({
      id: "fk" + (relations.length + 1),
      source: source.id,
      target: target.id,
      column: target.name + "_id",
      fkName: "fk_" + source.name + "_" + target.name + "_" + (relations.length + 1),
    });
  }

  const totalColumns = tables.reduce((s, t) => s + t.columns.length, 0);
  window.ER_FIXTURE = {
    seed: 42,
    tables,
    relations,
    stats: {
      tables: tables.length,
      relations: relations.length,
      columns: totalColumns,
      domains: DOMAINS.length,
    },
  };
})();
