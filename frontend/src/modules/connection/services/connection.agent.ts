import type { Connection, ConnectionConfig } from "../types/connection.types";

export class MockConnectionService {
  private connections: Connection[] = [];
  private nextId = 1;

  async list(): Promise<Connection[]> {
    return this.connections;
  }

  async get(id: string): Promise<Connection | null> {
    return this.connections.find((c) => c.id === id) ?? null;
  }

  async create(config: ConnectionConfig, _password: string): Promise<Connection> {
    const now = new Date().toISOString();
    const connection: Connection = {
      id: String(this.nextId++),
      ...config,
      createdAt: now,
      updatedAt: now,
    };
    this.connections.push(connection);
    return connection;
  }

  async update(id: string, config: ConnectionConfig, _password?: string): Promise<void> {
    const index = this.connections.findIndex((c) => c.id === id);
    if (index === -1) throw new Error(`Connection ${id} not found`);
    this.connections[index] = {
      ...this.connections[index],
      ...config,
      updatedAt: new Date().toISOString(),
    };
  }

  async delete(id: string): Promise<void> {
    this.connections = this.connections.filter((c) => c.id !== id);
  }

  async test(_config: ConnectionConfig, _password: string): Promise<void> {}

  async connect(id: string): Promise<void> {
    if (!this.connections.find((c) => c.id === id)) {
      throw new Error(`Connection ${id} not found`);
    }
  }

  async disconnect(_id: string): Promise<void> {}
}
