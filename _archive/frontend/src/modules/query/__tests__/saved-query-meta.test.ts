import { beforeEach, describe, expect, it } from "vitest";

import { useSavedQueryMetaStore } from "../stores/saved-query-meta.store";

function resetStore() {
  useSavedQueryMetaStore.setState({ meta: {} });
}

describe("SavedQueryMetaStore", () => {
  beforeEach(() => {
    resetStore();
  });

  it("returns default meta for unknown id", () => {
    const meta = useSavedQueryMetaStore.getState().getMeta("unknown");
    expect(meta).toEqual({ tags: [], favorite: false });
  });

  it("toggles favorite on", () => {
    useSavedQueryMetaStore.getState().toggleFavorite("q1");
    expect(useSavedQueryMetaStore.getState().isFavorite("q1")).toBe(true);
  });

  it("toggles favorite off", () => {
    useSavedQueryMetaStore.getState().toggleFavorite("q1");
    useSavedQueryMetaStore.getState().toggleFavorite("q1");
    expect(useSavedQueryMetaStore.getState().isFavorite("q1")).toBe(false);
  });

  it("isolates favorites between ids", () => {
    useSavedQueryMetaStore.getState().toggleFavorite("q1");
    expect(useSavedQueryMetaStore.getState().isFavorite("q1")).toBe(true);
    expect(useSavedQueryMetaStore.getState().isFavorite("q2")).toBe(false);
  });

  it("adds a tag", () => {
    useSavedQueryMetaStore.getState().addTag("q1", "important");
    const meta = useSavedQueryMetaStore.getState().getMeta("q1");
    expect(meta.tags).toEqual(["important"]);
  });

  it("does not add duplicate tags", () => {
    useSavedQueryMetaStore.getState().addTag("q1", "important");
    useSavedQueryMetaStore.getState().addTag("q1", "important");
    const meta = useSavedQueryMetaStore.getState().getMeta("q1");
    expect(meta.tags).toEqual(["important"]);
  });

  it("removes a tag", () => {
    useSavedQueryMetaStore.getState().addTag("q1", "important");
    useSavedQueryMetaStore.getState().addTag("q1", "draft");
    useSavedQueryMetaStore.getState().removeTag("q1", "important");
    const meta = useSavedQueryMetaStore.getState().getMeta("q1");
    expect(meta.tags).toEqual(["draft"]);
  });

  it("sets tags", () => {
    useSavedQueryMetaStore.getState().setTags("q1", ["a", "b"]);
    const meta = useSavedQueryMetaStore.getState().getMeta("q1");
    expect(meta.tags).toEqual(["a", "b"]);
  });

  it("collects all unique tags", () => {
    useSavedQueryMetaStore.getState().addTag("q1", "alpha");
    useSavedQueryMetaStore.getState().addTag("q1", "beta");
    useSavedQueryMetaStore.getState().addTag("q2", "alpha");
    useSavedQueryMetaStore.getState().addTag("q2", "gamma");
    const allTags = useSavedQueryMetaStore.getState().getAllTags();
    expect(allTags).toEqual(["alpha", "beta", "gamma"]);
  });
});
