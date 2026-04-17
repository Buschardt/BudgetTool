import { HLEDGER_FIELDS } from '../../types/rules';
import type { FieldAssignment } from '../../types/rules';
import './AssignmentRow.css';

interface Props {
  assignment: FieldAssignment;
  onChange: (a: FieldAssignment) => void;
  onRemove: () => void;
  /** CSV field names from the fields directive (for hints in value placeholder) */
  csvFields?: string[];
}

export function AssignmentRow({ assignment, onChange, onRemove, csvFields }: Props) {
  const placeholder =
    csvFields && csvFields.length > 0
      ? `e.g. expenses:food or %${csvFields.find(f => f) ?? '1'}`
      : 'e.g. expenses:food or %1';

  return (
    <div className="assignment-row">
      <select
        className="assignment-field-select"
        value={assignment.field}
        onChange={e => onChange({ ...assignment, field: e.target.value })}
      >
        <option value="">-- field --</option>
        {HLEDGER_FIELDS.map(f => (
          <option key={f} value={f}>
            {f}
          </option>
        ))}
      </select>
      <span className="assignment-eq">=</span>
      <input
        type="text"
        className="assignment-value-input"
        placeholder={placeholder}
        value={assignment.value}
        onChange={e => onChange({ ...assignment, value: e.target.value })}
      />
      <button type="button" className="assignment-remove-btn" onClick={onRemove} title="Remove">
        ×
      </button>
    </div>
  );
}
