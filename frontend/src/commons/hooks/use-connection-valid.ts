import { useConnectionList } from "@/modules/connection/queries/connection.queries";

export function useConnectionValid(connectionId: string | null): boolean {
  const connections = useConnectionList();
  if (connectionId === null) return true;
  if (!connections.data) return true;
  return connections.data.some((c) => c.id === connectionId);
}
