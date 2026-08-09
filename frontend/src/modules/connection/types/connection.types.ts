export type DriverType = "postgres" | "sqlite";

export type SslMode = "disable" | "require" | "verify-ca" | "verify-full";

export interface SshTunnelConfig {
  host: string;
  port: number;
  user: string;
  privateKeyPath: string;
  password?: string;
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
  sshTunnel?: SshTunnelConfig;
  queryTimeoutMs?: number;
  maxRows?: number;
  createdAt: string;
  updatedAt: string;
  color?: string;
  tags?: string[];
  group?: string;
  favorite?: boolean;
  readonly?: boolean;
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
  color?: string;
  tags?: string[];
  group?: string;
  favorite?: boolean;
  readonly?: boolean;
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
  color?: string;
  tags?: string[];
  group?: string;
  favorite?: boolean;
  readonly?: boolean;
}

export type ConnectionStatus =
  "connected" | "disconnected" | "connecting" | "reconnecting" | "error";
