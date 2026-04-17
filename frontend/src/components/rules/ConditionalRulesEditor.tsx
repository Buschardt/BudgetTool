import type { ConditionalRule } from '../../types/rules';
import { ConditionalRuleCard } from './ConditionalRuleCard';
import './ConditionalRulesEditor.css';

interface Props {
  conditionals: ConditionalRule[];
  onChange: (conditionals: ConditionalRule[]) => void;
  csvFields: string[];
}

function emptyBlockRule(): ConditionalRule {
  return {
    type: 'block',
    matchGroups: [{ matchers: [{ pattern: '', negate: false }] }],
    assignments: [{ field: 'account2', value: '' }],
    skip: false,
    end: false,
  };
}

function emptyTableRule(): ConditionalRule {
  return {
    type: 'table',
    tableFields: ['account2'],
    tableRows: [],
  };
}

export function ConditionalRulesEditor({ conditionals, onChange, csvFields }: Props) {
  function update(index: number, rule: ConditionalRule) {
    const next = [...conditionals];
    next[index] = rule;
    onChange(next);
  }

  function remove(index: number) {
    onChange(conditionals.filter((_, i) => i !== index));
  }

  function moveUp(index: number) {
    if (index === 0) return;
    const next = [...conditionals];
    [next[index - 1], next[index]] = [next[index], next[index - 1]];
    onChange(next);
  }

  function moveDown(index: number) {
    if (index === conditionals.length - 1) return;
    const next = [...conditionals];
    [next[index], next[index + 1]] = [next[index + 1], next[index]];
    onChange(next);
  }

  return (
    <section className="rule-section">
      <h2 className="rule-section-title">Conditional Rules</h2>
      <p className="rule-section-desc">
        Rules are applied in order. Use <strong>Conditions</strong> style for complex multi-field
        matching, or <strong>Table</strong> style to quickly map many patterns to account names.
        Match groups within a rule are OR-combined; matchers within a group are AND-combined.
      </p>

      {conditionals.length === 0 && (
        <p className="cre-empty">
          No conditional rules yet. Add one to categorize transactions automatically.
        </p>
      )}

      <div className="cre-list">
        {conditionals.map((rule, i) => (
          <ConditionalRuleCard
            key={i}
            rule={rule}
            index={i}
            total={conditionals.length}
            csvFields={csvFields}
            onChange={r => update(i, r)}
            onRemove={() => remove(i)}
            onMoveUp={() => moveUp(i)}
            onMoveDown={() => moveDown(i)}
          />
        ))}
      </div>

      <div className="cre-add-buttons">
        <button type="button" className="cre-add-btn" onClick={() => onChange([...conditionals, emptyBlockRule()])}>
          + Add condition rule
        </button>
        <button type="button" className="cre-add-btn cre-add-btn--table" onClick={() => onChange([...conditionals, emptyTableRule()])}>
          + Add table rule
        </button>
      </div>
    </section>
  );
}
