import type { ConnectionInput, DbEngine } from "@/types";

export type ConnectionDraft = {
  name: string;
  engine: DbEngine;
  path: string;
  host: string;
  port: string;
  database: string;
  username: string;
  password: string;
};

export const defaultConnectionDraft: ConnectionDraft = {
  name: "Local SQLite",
  engine: "sqlite",
  path: "./db-pro.sqlite",
  host: "localhost",
  port: "5432",
  database: "postgres",
  username: "postgres",
  password: "",
};

export function patchDraftForEngine(
  draft: ConnectionDraft,
  engine: DbEngine,
): ConnectionDraft {
  return {
    ...draft,
    engine,
    port: engine === "mysql" ? "3306" : "5432",
  };
}

export function toConnectionInput(
  draft: ConnectionDraft,
  id?: string,
): ConnectionInput {
  const payload: ConnectionInput = {
    id,
    name: draft.name.trim(),
    engine: draft.engine,
  };

  if (draft.engine === "sqlite") {
    payload.path = draft.path.trim() || ":memory:";
    return payload;
  }

  const parsedPort = Number.parseInt(draft.port, 10);
  payload.host = draft.host.trim();
  if (!Number.isNaN(parsedPort) && parsedPort > 0) {
    payload.port = parsedPort;
  }
  payload.database = draft.database.trim();
  payload.username = draft.username.trim();
  if (draft.password.length > 0) {
    payload.password = draft.password;
  }

  return payload;
}

export function fromConnectionInput(input: ConnectionInput): ConnectionDraft {
  if (input.engine === "sqlite") {
    return {
      name: input.name,
      engine: "sqlite",
      path: input.path ?? "./db-pro.sqlite",
      host: "localhost",
      port: "5432",
      database: "postgres",
      username: "postgres",
      password: "",
    };
  }

  return {
    name: input.name,
    engine: input.engine,
    path: "./db-pro.sqlite",
    host: input.host ?? "localhost",
    port: (input.port ?? (input.engine === "mysql" ? 3306 : 5432)).toString(),
    database: input.database ?? "",
    username: input.username ?? "",
    password: "",
  };
}
