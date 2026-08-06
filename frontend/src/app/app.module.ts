import { DIContainer } from "@/commons/di/container";
import { SERVICE_NAMES } from "@/commons/di/registry";
import { createConnectionService } from "@/modules/connection/services/connection.service";
import { createDataGridService } from "@/modules/data-grid/services/data-grid.service";
import { createExportService } from "@/modules/export/services/export.service";
import { createQueryService } from "@/modules/query/services/query.service";
import { createSchemaService } from "@/modules/schema/services/schema.service";

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

  container.register(SERVICE_NAMES.SCHEMA_SERVICE, () =>
    createSchemaService(),
  );

  container.register(SERVICE_NAMES.DATA_GRID_SERVICE, () =>
    createDataGridService(),
  );

  container.register(SERVICE_NAMES.EXPORT_SERVICE, () =>
    createExportService(),
  );

  container.freeze();
  bootstrapped = true;
  return container;
}

export { container };
