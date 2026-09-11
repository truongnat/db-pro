/*
 * ER benchmark fixture generator — shared by both harnesses.
 *
 * Deterministic (mulberry32 seeded PRNG): same options => identical data on
 * every load, so harness A (Cytoscape) and harness B (React Flow) always
 * benchmark the exact same graph.
 *
 * Usage:
 *   window.ER_FIXTURE = generateERFixture({ ... })   // or
 *   window.ER_FIXTURE = generateERFixture.presets["500"]
 *
 * Presets (locked in docs/plans/active/ui-foundation-scale-hardening):
 *   A 100  tables  ~150  relations  ~1,200 columns
 *   B 500  tables  ~900  relations  ~7,500 columns
 *   C 1000 tables  ~2000 relations  ~15,000 columns
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

  const COLUMN_TYPES = [
    "bigint", "integer", "varchar(255)", "text", "timestamp",
    "timestamptz", "boolean", "numeric(12,2)", "uuid", "date", "jsonb", "double precision",
  ];
  const COMMON_COLS = ["id", "created_at", "updated_at", "status", "is_active", "version", "tenant_id", "notes", "metadata"];

  const DOMAIN_NAMES = [
    "customers", "orders", "products", "payments", "accounts",
    "shipping", "inventory", "billing", "hr", "crm",
    "marketing", "support", "analytics", "content", "subscriptions",
    "warehouse", "finance", "logistics", "catalog", "reports",
  ];
  const TABLE_SUFFIX = ["", "", "", "_history", "_archive", "_audit", "_log", "_draft"];

  /**
   * @param {object} opts
   * @param {number} opts.tables      total tables (must divide by domains)
   * @param {number} opts.relations   target relation count
   * @param {number} opts.domains     number of domains (<= DOMAIN_NAMES.length)
   * @param {number} opts.colMin      columns per table (min)
   * @param {number} opts.colMax      columns per table (max)
   * @param {number} [opts.seed]      PRNG seed (default 42)
   */
  function generateERFixture(opts) {
    const seed = opts.seed ?? 42;
    const rand = mulberry32(seed);
    const pick = (arr) => arr[Math.floor(rand() * arr.length)];
    const randInt = (min, max) => min + Math.floor(rand() * (max - min + 1));

    const domains = DOMAIN_NAMES.slice(0, opts.domains);
    const perDomain = Math.round(opts.tables / opts.domains);

    const tables = [];
    const relations = [];
    let tableSeq = 1;

    for (const domain of domains) {
      for (let i = 0; i < perDomain; i++) {
        const id = "t" + String(tableSeq++).padStart(4, "0");
        const name = i === 0 ? domain : domain + "_" + i + pick(TABLE_SUFFIX);
        const nCols = randInt(opts.colMin, opts.colMax);
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
        tables.push({ id, name, domain, columns });
      }
    }

    // Relations: 65% intra-domain, 35% cross-domain; each relation adds one FK column.
    const seenPairs = new Set();
    let guard = 0;
    while (relations.length < opts.relations && guard++ < opts.relations * 40) {
      const source = tables[Math.floor(rand() * tables.length)];
      let target;
      if (rand() < 0.65) {
        const sameDomain = tables.filter((t) => t.domain === source.domain && t.id !== source.id);
        if (!sameDomain.length) continue;
        target = sameDomain[Math.floor(rand() * sameDomain.length)];
      } else {
        const others = tables.filter((t) => t.domain !== source.domain);
        if (!others.length) continue;
        target = others[Math.floor(rand() * others.length)];
      }
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

    return {
      seed,
      tables,
      relations,
      stats: {
        tables: tables.length,
        relations: relations.length,
        columns: tables.reduce((s, t) => s + t.columns.length, 0),
        domains: domains.length,
      },
    };
  }

  generateERFixture.presets = {
    "100": { tables: 100, relations: 150, domains: 10, colMin: 8, colMax: 16, seed: 42 },
    "500": { tables: 500, relations: 900, domains: 20, colMin: 10, colMax: 20, seed: 42 },
    "1000": { tables: 1000, relations: 2000, domains: 20, colMin: 10, colMax: 20, seed: 42 },
  };

  window.generateERFixture = generateERFixture;
})();
