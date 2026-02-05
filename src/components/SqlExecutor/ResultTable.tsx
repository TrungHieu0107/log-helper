import { useMemo } from "react";
import {
  useReactTable,
  getCoreRowModel,
  flexRender,
  createColumnHelper,
  type ColumnDef,
} from "@tanstack/react-table";
import type { CellValue, QueryResult } from "../../types";

interface ResultTableProps {
  result: QueryResult;
}

function cellValueToString(cell: CellValue): string {
  if (cell === "Null") return "NULL";
  if (typeof cell === "object") {
    if ("Text" in cell) return cell.Text;
    if ("Int" in cell) return cell.Int.toString();
    if ("Float" in cell) return cell.Float.toString();
    if ("Bool" in cell) return cell.Bool.toString();
    if ("DateTime" in cell) return cell.DateTime;
    if ("Binary" in cell) return cell.Binary;
  }
  return "?";
}

function isNull(cell: CellValue): boolean {
  return cell === "Null";
}

type RowData = Record<string, CellValue>;

export default function ResultTable({ result }: ResultTableProps) {
  const columnHelper = createColumnHelper<RowData>();

  const columns: ColumnDef<RowData, CellValue>[] = useMemo(
    () =>
      result.columns.map((colName, colIdx) =>
        columnHelper.accessor((row) => row[`col_${colIdx}`], {
          id: `col_${colIdx}`,
          header: () => colName,
          cell: (info) => {
            const val = info.getValue();
            if (!val || isNull(val)) {
              return <span className="cell-null">NULL</span>;
            }
            return cellValueToString(val);
          },
          size: 150,
          minSize: 60,
        }),
      ),
    [result.columns, columnHelper],
  );

  const data: RowData[] = useMemo(
    () =>
      result.rows.map((row) => {
        const obj: RowData = {};
        for (let i = 0; i < result.columns.length; i++) {
          obj[`col_${i}`] = row[i] ?? "Null";
        }
        return obj;
      }),
    [result.rows, result.columns],
  );

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    columnResizeMode: "onChange",
  });

  return (
    <div>
      <div className="meta-info">
        {result.columns.length} columns, {result.rows.length} rows
      </div>
      <div className="result-table-container">
        <table
          className="result-table"
          style={{ width: table.getCenterTotalSize() }}
        >
          <thead>
            {table.getHeaderGroups().map((headerGroup) => (
              <tr key={headerGroup.id}>
                {headerGroup.headers.map((header) => (
                  <th
                    key={header.id}
                    style={{ width: header.getSize() }}
                  >
                    {header.isPlaceholder
                      ? null
                      : flexRender(
                          header.column.columnDef.header,
                          header.getContext(),
                        )}
                    <div
                      className={`resizer ${header.column.getIsResizing() ? "isResizing" : ""}`}
                      onMouseDown={header.getResizeHandler()}
                      onTouchStart={header.getResizeHandler()}
                    />
                  </th>
                ))}
              </tr>
            ))}
          </thead>
          <tbody>
            {table.getRowModel().rows.map((row) => (
              <tr key={row.id}>
                {row.getVisibleCells().map((cell) => (
                  <td key={cell.id} style={{ width: cell.column.getSize() }}>
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
