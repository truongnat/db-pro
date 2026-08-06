import { apiInvoke } from "@/commons/utils/api";

import type { Connection, ConnectionConfig, SshTunnelConfig } from "../types/connection.types";

export class ConnectionService {
  async list(): Promise<Connection[]> {
    return apiInvoke<Connection[]>("list_connections");
  }

  async get(id: string): Promise<Connection | null> {
    return apiInvoke<Connection | null>("get_connection", { id });
  }

  async create(config: ConnectionConfig, password: string): Promise<Connection> {
    return apiInvoke<Connection>("create_connection", { config, password });
  }

  async update(id: string, config: ConnectionConfig, password?: string): Promise<void> {
    return apiInvoke<void>("update_connection", { id, config, password });
  }

  async delete(id: string): Promise<void> {
    return apiInvoke<void>("delete_connection", { id });
  }

  async test(config: ConnectionConfig, password: string, connectionId?: string): Promise<void> {
    return apiInvoke<void>("test_connection", { config, password, connectionId });
  }

  async connect(id: string): Promise<void> {
    return apiInvoke<void>("connect", { id });
  }

  async disconnect(id: string): Promise<void> {
    return apiInvoke<void>("disconnect", { id });
  }

  async testSshTunnel(config: SshTunnelConfig): Promise<void> {
    return apiInvoke<void>("test_ssh_tunnel", { config });
  }
}

export function createConnectionService(): ConnectionService {
  return new ConnectionService();
}
