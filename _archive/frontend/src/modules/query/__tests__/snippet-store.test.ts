import { beforeEach, describe, expect, it } from "vitest";

import { expandSnippet, useSnippetStore } from "../stores/snippet.store";
import { BUILT_IN_SNIPPETS } from "../types/snippet.types";

function resetStore() {
  useSnippetStore.setState({ custom: [] });
}

describe("SnippetStore", () => {
  beforeEach(() => {
    resetStore();
  });

  it("returns built-in snippets via getAll", () => {
    const all = useSnippetStore.getState().getAll();
    expect(all.length).toBe(BUILT_IN_SNIPPETS.length);
    expect(all.every((s) => s.builtIn)).toBe(true);
  });

  it("adds a custom snippet", () => {
    useSnippetStore.getState().addSnippet("mysel", "My Select", "SELECT 1;");
    const all = useSnippetStore.getState().getAll();
    expect(all.length).toBe(BUILT_IN_SNIPPETS.length + 1);
    const custom = all.find((s) => s.trigger === "mysel");
    expect(custom).toBeDefined();
    expect(custom!.builtIn).toBe(false);
  });

  it("replaces custom snippet with same trigger", () => {
    useSnippetStore.getState().addSnippet("mysel", "First", "SELECT 1;");
    useSnippetStore.getState().addSnippet("mysel", "Second", "SELECT 2;");
    const all = useSnippetStore.getState().getAll();
    const customs = all.filter((s) => s.trigger === "mysel");
    expect(customs).toHaveLength(1);
    expect(customs[0].label).toBe("Second");
  });

  it("removes a custom snippet", () => {
    useSnippetStore.getState().addSnippet("mysel", "My Select", "SELECT 1;");
    useSnippetStore.getState().removeSnippet("mysel");
    const all = useSnippetStore.getState().getAll();
    expect(all.find((s) => s.trigger === "mysel")).toBeUndefined();
  });

  it("updates a custom snippet", () => {
    useSnippetStore.getState().addSnippet("mysel", "Old", "SELECT 1;");
    useSnippetStore.getState().updateSnippet("mysel", "New", "SELECT 2;");
    const all = useSnippetStore.getState().getAll();
    const found = all.find((s) => s.trigger === "mysel");
    expect(found!.label).toBe("New");
    expect(found!.body).toBe("SELECT 2;");
  });

  it("searches by trigger", () => {
    const results = useSnippetStore.getState().search("sel");
    expect(results.length).toBeGreaterThan(0);
    expect(results.some((s) => s.trigger === "sel")).toBe(true);
  });

  it("searches by label", () => {
    const results = useSnippetStore.getState().search("CREATE TABLE");
    expect(results.some((s) => s.trigger === "ct")).toBe(true);
  });

  it("returns all when search is empty", () => {
    useSnippetStore.getState().addSnippet("custom", "Custom", "SELECT 1;");
    const results = useSnippetStore.getState().search("");
    expect(results.length).toBe(BUILT_IN_SNIPPETS.length + 1);
  });
});

describe("expandSnippet", () => {
  it("replaces $cursor with empty string", () => {
    expect(expandSnippet("SELECT * FROM $cursor;")).toBe("SELECT * FROM ;");
  });

  it("replaces multiple $cursor occurrences", () => {
    expect(expandSnippet("$cursor JOIN $cursor")).toBe(" JOIN ");
  });

  it("returns body unchanged when no $cursor", () => {
    expect(expandSnippet("SELECT 1;")).toBe("SELECT 1;");
  });
});
