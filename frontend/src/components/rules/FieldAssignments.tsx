import type { FieldAssignment } from '../../types/rules';
import { AssignmentRow } from './AssignmentRow';
import './FieldAssignments.css';

interface Props {
  assignments: FieldAssignment[];
  onChange: (assignments: FieldAssignment[]) => void;
  csvFields?: string[];
}

export function FieldAssignments({ assignments, onChange, csvFields }: Props) {
  function update(index: number, a: FieldAssignment) {
    const next = [...assignments];
    next[index] = a;
    onChange(next);
  }

  function remove(index: number) {
    onChange(assignments.filter((_, i) => i !== index));
  }

  function add() {
    onChange([...assignments, { field: 'account1', value: '' }]);
  }

  return (
    <section className="rule-section">
      <h2 className="rule-section-title">Field Assignments</h2>
      <p className="rule-section-desc">
        Set hledger field values that apply to every transaction (not conditional). Use{' '}
        <code>%columnname</code> or <code>%N</code> to interpolate CSV column values.
      </p>

      {assignments.length === 0 && (
        <p className="fa-empty">No assignments. Add one below.</p>
      )}

      <div className="fa-list">
        {assignments.map((a, i) => (
          <AssignmentRow
            key={i}
            assignment={a}
            onChange={v => update(i, v)}
            onRemove={() => remove(i)}
            csvFields={csvFields}
          />
        ))}
      </div>

      <button type="button" className="fa-add-btn" onClick={add}>
        + Add assignment
      </button>
    </section>
  );
}
