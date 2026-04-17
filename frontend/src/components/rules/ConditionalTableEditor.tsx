import type { ConditionalRule, TableRow } from '../../types/rules';
import { HLEDGER_FIELDS } from '../../types/rules';
import './ConditionalTableEditor.css';

interface Props {
  rule: ConditionalRule;
  onChange: (r: ConditionalRule) => void;
}

export function ConditionalTableEditor({ rule, onChange }: Props) {
  const fields = rule.tableFields ?? [];
  const rows = rule.tableRows ?? [];

  function updateField(colIdx: number, value: string) {
    const next = [...fields];
    next[colIdx] = value;
    onChange({ ...rule, tableFields: next });
  }

  function addColumn() {
    onChange({ ...rule, tableFields: [...fields, ''] });
  }

  function removeColumn(colIdx: number) {
    const nextFields = fields.filter((_, i) => i !== colIdx);
    const nextRows = rows.map(row => ({
      ...row,
      values: row.values.filter((_, i) => i !== colIdx),
    }));
    onChange({ ...rule, tableFields: nextFields, tableRows: nextRows });
  }

  function updateRowPattern(rowIdx: number, pattern: string) {
    const nextRows = [...rows];
    nextRows[rowIdx] = { ...nextRows[rowIdx], pattern };
    onChange({ ...rule, tableRows: nextRows });
  }

  function updateRowValue(rowIdx: number, colIdx: number, value: string) {
    const nextRows = rows.map((row, ri) => {
      if (ri !== rowIdx) return row;
      const nextValues = [...(row.values ?? [])];
      nextValues[colIdx] = value;
      return { ...row, values: nextValues };
    });
    onChange({ ...rule, tableRows: nextRows });
  }

  function addRow() {
    onChange({
      ...rule,
      tableRows: [...rows, { pattern: '', values: fields.map(() => '') }],
    });
  }

  function removeRow(rowIdx: number) {
    onChange({ ...rule, tableRows: rows.filter((_, i) => i !== rowIdx) });
  }

  function cellValue(row: TableRow, colIdx: number): string {
    return row.values?.[colIdx] ?? '';
  }

  return (
    <div className="cte">
      <p className="cte-desc">
        Each row maps a regex pattern to field values. The pattern is matched against the whole CSV
        record.
      </p>

      <div className="cte-table-wrapper">
        <table className="cte-table">
          <thead>
            <tr>
              <th className="cte-th cte-th--pattern">Pattern (regex)</th>
              {fields.map((f, ci) => (
                <th key={ci} className="cte-th">
                  <select
                    className="cte-field-select"
                    value={f}
                    onChange={e => updateField(ci, e.target.value)}
                  >
                    <option value="">-- field --</option>
                    {HLEDGER_FIELDS.map(hf => (
                      <option key={hf} value={hf}>
                        {hf}
                      </option>
                    ))}
                  </select>
                  <button
                    type="button"
                    className="cte-remove-col-btn"
                    onClick={() => removeColumn(ci)}
                    title="Remove column"
                  >
                    ×
                  </button>
                </th>
              ))}
              <th className="cte-th cte-th--actions">
                <button type="button" className="cte-add-col-btn" onClick={addColumn}>
                  + Col
                </button>
              </th>
            </tr>
          </thead>
          <tbody>
            {rows.length === 0 && (
              <tr>
                <td
                  colSpan={fields.length + 2}
                  className="cte-empty-row"
                >
                  No rows yet. Add rows below.
                </td>
              </tr>
            )}
            {rows.map((row, ri) => (
              <tr key={ri}>
                <td>
                  <input
                    type="text"
                    className="cte-pattern-input"
                    placeholder="regex…"
                    value={row.pattern}
                    onChange={e => updateRowPattern(ri, e.target.value)}
                  />
                </td>
                {fields.map((_, ci) => (
                  <td key={ci}>
                    <input
                      type="text"
                      className="cte-value-input"
                      placeholder="value…"
                      value={cellValue(row, ci)}
                      onChange={e => updateRowValue(ri, ci, e.target.value)}
                    />
                  </td>
                ))}
                <td>
                  <button
                    type="button"
                    className="cte-remove-row-btn"
                    onClick={() => removeRow(ri)}
                    title="Remove row"
                  >
                    ×
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <button type="button" className="cte-add-row-btn" onClick={addRow}>
        + Add row
      </button>
    </div>
  );
}
