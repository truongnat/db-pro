import { DIContainer } from "@/commons/di/container";
import { SERVICE_NAMES } from "@/commons/di/registry";
import { createConnectionService } from "@/modules/connection/services/connection.service";
import { createQueryService } from "@/modules/query/services/query.service";

const container = new DIContainer();
let bootstrapped = false;

export async function bootstrapServices(): Promise<DIContainer> {
  if (bootstrapped) return container;

  container.register(SERVICE_NAMES.CONNECTION_SERVICE, () =>
    createConnectionService(),
  );

  container.register(SERVICE_NAMES.QUERY_SERVICE, () =>
    createQueryService(),
  );

  container.register(SERVICE_NAMES.SCHEMA_SERVICE, () => {
    throw new Error("SchemaService not yet implemented (Phase 9)");
  });

  container.register(SERVICE_NAMES.EXPORT_SERVICE, () => {
    throw new Error("ExportService not yet implemented (Phase 11)");
  });

  container.freeze();
  bootstrapped = true;
  return container;
}

export { container };
