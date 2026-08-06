export type DriverType = "postgres" | "sqlite";

export type SslMode = "disable" | "require" | "verify-ca" | "verify-full";

export interface SshTunnelConfig {
  host: string;
  port: number;
  user: string;
  privateKeyPath: string;
}

export interface Connection {
  id: string;
  name: string;
  host: string;
  port: number;
  database: string;
  username: string;
  driver: DriverType;
  sslMode: SslMode;
  createdAt: string;
  updatedAt: string;
}

export interface ConnectionConfig {
  name: string;
  host: string;
  port: number;
  database: string;
  username: string;
  driver: DriverType;
  sslMode: SslMode;
  sshTunnel?: SshTunnelConfig;
  queryTimeoutMs: number;
  maxRows: number;
}

export interface ConnectionFormData {
  name: string;
  host: string;
  port: number;
  database: string;
  username: string;
  driver: DriverType;
  sslMode: SslMode;
  sshTunnel?: SshTunnelConfig;
  queryTimeoutMs: number;
  maxRows: number;
}

export type ConnectionStatus = "connected" | "disconnected" | "connecting" | "error";
