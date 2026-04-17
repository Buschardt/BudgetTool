import './FieldsMapping.css';

interface Props {
  fields: string[];
  onChange: (fields: string[]) => void;
}

export function FieldsMapping({ fields, onChange }: Props) {
  function updateField(index: number, value: string) {
    const next = [...fields];
    next[index] = value;
    onChange(next);
  }

  function addColumn() {
    onChange([...fields, '']);
  }

  function removeColumn(index: number) {
    onChange(fields.filter((_, i) => i !== index));
  }

  return (
    <section className="rule-section">
      <h2 className="rule-section-title">CSV Column Mapping</h2>
      <p className="rule-section-desc">
        Name the columns in your CSV file. Use standard hledger field names (date, description,
        amount, etc.) to auto-assign them, or any name you like to reference them in rules below
        with <code>%columnname</code>. Leave a column blank to ignore it.
      </p>

      {fields.length === 0 ? (
        <p className="fields-empty">No columns defined. Add columns to name your CSV fields.</p>
      ) : (
        <div className="fields-list">
          {fields.map((f, i) => (
            <div key={i} className="fields-row">
              <span className="fields-col-num">Col {i + 1}</span>
              <input
                type="text"
                className="fields-input"
                placeholder="(ignore)"
                value={f}
                onChange={e => updateField(i, e.target.value)}
              />
              <button
                type="button"
                className="fields-remove-btn"
                onClick={() => removeColumn(i)}
                title="Remove column"
              >
                ×
              </button>
            </div>
          ))}
        </div>
      )}

      <button type="button" className="fields-add-btn" onClick={addColumn}>
        + Add column
      </button>
    </section>
  );
}
