import type {
  FetchRowsRequest,
  FetchRowsResult,
  MutateRowRequest,
  MutateRowResult,
} from "../types/data-grid.types";

const MOCK_COLUMNS = [
  { name: "id", dataType: "INTEGER", nullable: false },
  { name: "name", dataType: "TEXT", nullable: false },
  { name: "email", dataType: "TEXT", nullable: true },
  { name: "age", dataType: "INTEGER", nullable: true },
];

function makeRow(id: number) {
  return [
    { type: "int64" as const, value: id },
    { type: "text" as const, value: `User ${id}` },
    { type: "text" as const, value: `user${id}@test.com` },
    { type: "int64" as const, value: 20 + id },
  ];
}

const ALL_ROWS = Array.from({ length: 87 }, (_, i) => makeRow(i + 1));

export class MockDataGridService {
  async fetchRows(_connectionId: string, request: FetchRowsRequest): Promise<FetchRowsResult> {
    const offset = (request.page - 1) * request.pageSize;
    const rows = ALL_ROWS.slice(offset, offset + request.pageSize);
    return {
      columns: MOCK_COLUMNS,
      rows,
      totalCount: ALL_ROWS.length,
      durationMs: 3,
    };
  }

  async insertRow(_connectionId: string, _request: MutateRowRequest): Promise<MutateRowResult> {
    return { affectedRows: 1 };
  }

  async updateRow(_connectionId: string, _request: MutateRowRequest): Promise<MutateRowResult> {
    return { affectedRows: 1 };
  }

  async deleteRow(_connectionId: string, _request: MutateRowRequest): Promise<MutateRowResult> {
    return { affectedRows: 1 };
  }
}

export function createMockDataGridService(): MockDataGridService {
  return new MockDataGridService();
}
