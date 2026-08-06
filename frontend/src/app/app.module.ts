import { DIContainer } from "@/commons/di/container";
import { SERVICE_NAMES } from "@/commons/di/registry";

const container = new DIContainer();

export async function bootstrapServices(): Promise<DIContainer> {
  container.register(SERVICE_NAMES.CONNECTION_SERVICE, () => {
    throw new Error("ConnectionService not yet implemented (Phase 7)");
  });

  container.register(SERVICE_NAMES.QUERY_SERVICE, () => {
    throw new Error("QueryService not yet implemented (Phase 8)");
  });

  container.register(SERVICE_NAMES.SCHEMA_SERVICE, () => {
    throw new Error("SchemaService not yet implemented (Phase 9)");
  });

  container.register(SERVICE_NAMES.EXPORT_SERVICE, () => {
    throw new Error("ExportService not yet implemented (Phase 11)");
  });

  container.freeze();
  return container;
}

export { container };
