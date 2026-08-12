import { describe, it, expect } from "vitest";
import { classifySchemaComplexity, computeSchemaComplexity } from "../renderer/er-graph-model";

describe("computeSchemaComplexity (locked P1 hard rule #5)", () => {
  it("implements complexity = tables + relations*0.7 + columns*0.08", () => {
    expect(computeSchemaComplexity({ tables: 10, relations: 10, columns: 100 })).toBeCloseTo(
      10 + 7 + 8,
    );
    expect(computeSchemaComplexity({ tables: 0, relations: 0, columns: 0 })).toBe(0);
  });

  it("reproduces the P1.8 benchmark fixture scores", () => {
    // A100: 100 + 150*0.7 + 1318*0.08 = 310.44
    expect(computeSchemaComplexity({ tables: 100, relations: 150, columns: 1318 })).toBeCloseTo(
      310.44,
      1,
    );
    // A500: 500 + 900*0.7 + 8406*0.08 = 1802.48
    expect(computeSchemaComplexity({ tables: 500, relations: 900, columns: 8406 })).toBeCloseTo(
      1802.48,
      1,
    );
    // A1000: 1000 + 2000*0.7 + 17065*0.08 = 3765.2
    expect(computeSchemaComplexity({ tables: 1000, relations: 2000, columns: 17065 })).toBeCloseTo(
      3765.2,
      1,
    );
  });

  it("weights columns even at a small per-column factor", () => {
    // Two schemas with the same table count — the denser one scores higher.
    const lean = computeSchemaComplexity({ tables: 50, relations: 0, columns: 50 });
    const dense = computeSchemaComplexity({ tables: 50, relations: 0, columns: 500 });
    expect(dense).toBeGreaterThan(lean);
  });
});

describe("classifySchemaComplexity (thresholds tuned from P1.8 evidence)", () => {
  it("classifies S/M/L/XL at the tuned boundaries", () => {
    expect(classifySchemaComplexity(0)).toBe("S");
    expect(classifySchemaComplexity(99)).toBe("S");
    expect(classifySchemaComplexity(100)).toBe("M");
    expect(classifySchemaComplexity(699)).toBe("M");
    expect(classifySchemaComplexity(700)).toBe("L");
    expect(classifySchemaComplexity(1999)).toBe("L");
    expect(classifySchemaComplexity(2000)).toBe("XL");
    expect(classifySchemaComplexity(4000)).toBe("XL");
  });

  it("maps the benchmark fixtures to the evidence-backed tiers", () => {
    // A100 (310.4) → M: React Flow full graph is excellent (60 fps, 1,143 DOM)
    // — no exploration UX, no canvas overview.
    expect(
      classifySchemaComplexity(
        computeSchemaComplexity({ tables: 100, relations: 150, columns: 1318 }),
      ),
    ).toBe("M");

    // A500 (1,802.5) → L: full graph on canvas immediately (opass-style focus
    // + highlight; React Flow full-graph overview drops to 34 fps).
    expect(
      classifySchemaComplexity(
        computeSchemaComplexity({ tables: 500, relations: 900, columns: 8406 }),
      ),
    ).toBe("L");

    // A1000 (3,765.2) → XL: full-graph React Flow is not viable (122 s layout).
    expect(
      classifySchemaComplexity(
        computeSchemaComplexity({ tables: 1000, relations: 2000, columns: 17065 }),
      ),
    ).toBe("XL");
  });

  it("keeps lean medium schemas below the exploration threshold", () => {
    // A 200-table schema with modest relations/columns stays M: the old
    // hardcoded `tables > 200` gate would have flipped this to large.
    expect(
      classifySchemaComplexity(
        computeSchemaComplexity({ tables: 200, relations: 150, columns: 1200 }),
      ),
    ).toBe("M");
  });
});
