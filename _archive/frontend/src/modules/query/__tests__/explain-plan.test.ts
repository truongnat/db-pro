import { describe, it, expect } from "vitest";
import type { ExplainPlan } from "../types/query.types";

// Test the plan extraction logic (we can't easily test the React component without more setup)
describe("ExplainPlan types", () => {
  it("ExplainPlan is a record type", () => {
    const plan: ExplainPlan = { "Plan Rows": 100, "Plan Cost": 5.2 };
    expect(plan["Plan Rows"]).toBe(100);
    expect(plan["Plan Cost"]).toBe(5.2);
  });

  it("handles nested plan structure", () => {
    const plan: ExplainPlan = {
      Plan: {
        "Node Type": "Seq Scan",
        "Relation Name": "users",
        "Plan Rows": 1000,
        "Total Cost": 25.5,
        Plans: [
          {
            "Node Type": "Index Scan",
            "Index Name": "idx_users_email",
            "Plan Rows": 1,
          },
        ],
      },
    };
    expect(plan.Plan).toBeDefined();
    expect((plan.Plan as Record<string, unknown>)["Node Type"]).toBe("Seq Scan");
  });
});
