import type { RulesConfigSummary } from '../../types/rules';
import './IncludesSelector.css';

interface Props {
  includes: number[];
  allConfigs: RulesConfigSummary[];
  currentId: number | null; // the config being edited (exclude from list)
  onChange: (includes: number[]) => void;
}

export function IncludesSelector({ includes, allConfigs, currentId, onChange }: Props) {
  const available = allConfigs.filter(c => c.id !== currentId);

  function toggle(id: number) {
    if (includes.includes(id)) {
      onChange(includes.filter(i => i !== id));
    } else {
      onChange([...includes, id]);
    }
  }

  if (available.length === 0) return null;

  return (
    <section className="rule-section">
      <h2 className="rule-section-title">Include Other Configs</h2>
      <p className="rule-section-desc">
        Include rules from other configurations. Their rules will be inlined at the end of this
        config (in the order listed below).
      </p>

      <div className="includes-list">
        {available.map(c => (
          <label key={c.id} className="includes-item">
            <input
              type="checkbox"
              checked={includes.includes(c.id)}
              onChange={() => toggle(c.id)}
            />
            <span className="includes-name">{c.name}</span>
            {c.description && (
              <span className="includes-desc">{c.description}</span>
            )}
          </label>
        ))}
      </div>
    </section>
  );
}
