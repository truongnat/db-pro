import {
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  Legend,
  Line,
  LineChart,
  Pie,
  PieChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";

import { renderCellValue } from "@/modules/query/types/query.types";

import type { ChartConfig } from "../state/data-grid.store";
import type { ColumnMeta, Row } from "../types/data-grid.types";

const COLORS = [
  "#3b82f6",
  "#10b981",
  "#f59e0b",
  "#ef4444",
  "#8b5cf6",
  "#ec4899",
  "#06b6d4",
  "#84cc16",
];

interface ChartViewProps {
  columns: ColumnMeta[];
  rows: Row[];
  config: ChartConfig;
}

export function ChartView({ columns, rows, config }: ChartViewProps) {
  const xIdx = columns.findIndex((c) => c.name === config.xColumn);
  const yIdx = columns.findIndex((c) => c.name === config.yColumn);

  if (xIdx === -1 || yIdx === -1) {
    return (
      <div className="flex items-center justify-center py-12 text-sm text-[var(--text-secondary)]">
        Invalid chart configuration
      </div>
    );
  }

  const data = rows.map((row) => ({
    name: renderCellValue(row[xIdx]),
    value: getNumericValue(row[yIdx]),
  }));

  if (data.length === 0) {
    return (
      <div className="flex items-center justify-center py-12 text-sm text-[var(--text-secondary)]">
        No data to chart
      </div>
    );
  }

  return (
    <div className="h-full w-full p-4">
      <ResponsiveContainer width="100%" height="100%">
        {config.type === "bar" ? (
          <BarChart data={data}>
            <CartesianGrid strokeDasharray="3 3" stroke="var(--border-default)" />
            <XAxis dataKey="name" tick={{ fontSize: 11 }} />
            <YAxis tick={{ fontSize: 11 }} />
            <Tooltip />
            <Legend />
            <Bar dataKey="value" fill="#3b82f6" />
          </BarChart>
        ) : config.type === "line" ? (
          <LineChart data={data}>
            <CartesianGrid strokeDasharray="3 3" stroke="var(--border-default)" />
            <XAxis dataKey="name" tick={{ fontSize: 11 }} />
            <YAxis tick={{ fontSize: 11 }} />
            <Tooltip />
            <Legend />
            <Line type="monotone" dataKey="value" stroke="#3b82f6" strokeWidth={2} dot={{ r: 3 }} />
          </LineChart>
        ) : (
          <PieChart>
            <Tooltip />
            <Legend />
            <Pie data={data} dataKey="value" nameKey="name" cx="50%" cy="50%" outerRadius={100}>
              {data.map((_, index) => (
                <Cell key={index} fill={COLORS[index % COLORS.length]} />
              ))}
            </Pie>
          </PieChart>
        )}
      </ResponsiveContainer>
    </div>
  );
}

function getNumericValue(cell: { type: string; value?: unknown }): number {
  if (cell.type === "int64" || cell.type === "float64") return cell.value as number;
  if (cell.type === "text") {
    const n = Number(cell.value);
    return Number.isFinite(n) ? n : 0;
  }
  return 0;
}
