import './DataTable.css';

export interface Column<T> {
  key: keyof T | string;
  header: string;
  align?: 'left' | 'right' | 'center';
  render?: (row: T) => React.ReactNode;
}

interface DataTableProps<T> {
  columns: Column<T>[];
  rows: T[];
  keyFn?: (row: T, index: number) => string | number;
  emptyMessage?: string;
}

export function DataTable<T extends object>({
  columns,
  rows,
  keyFn,
  emptyMessage = 'No data.',
}: DataTableProps<T>) {
  if (rows.length === 0) {
    return <div className="data-table-empty">{emptyMessage}</div>;
  }

  return (
    <div className="data-table-wrapper">
      <table className="data-table">
        <thead>
          <tr>
            {columns.map(col => (
              <th
                key={String(col.key)}
                className={`data-table-th data-table-th--${col.align ?? 'left'}`}
              >
                {col.header}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row, i) => (
            <tr key={keyFn ? keyFn(row, i) : i} className="data-table-row">
              {columns.map(col => {
                const value = col.render
                  ? col.render(row)
                  : String((row as Record<string, unknown>)[String(col.key)] ?? '');
                return (
                  <td
                    key={String(col.key)}
                    className={`data-table-td data-table-td--${col.align ?? 'left'}`}
                  >
                    {value}
                  </td>
                );
              })}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
