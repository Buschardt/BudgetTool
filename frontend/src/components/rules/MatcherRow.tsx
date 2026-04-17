import type { Matcher } from '../../types/rules';
import './MatcherRow.css';

interface Props {
  matcher: Matcher;
  onChange: (m: Matcher) => void;
  onRemove: () => void;
  csvFields: string[];
  isFirst: boolean; // first in its group — rendered without "AND" prefix
}

export function MatcherRow({ matcher, onChange, onRemove, csvFields, isFirst }: Props) {
  const namedFields = csvFields.filter(f => f.trim() !== '');

  return (
    <div className="matcher-row">
      {!isFirst && <span className="matcher-and-badge">AND</span>}

      {/* Field scope */}
      <select
        className="matcher-field-select"
        value={matcher.field ?? ''}
        onChange={e => onChange({ ...matcher, field: e.target.value || undefined })}
        title="Match against which field"
      >
        <option value="">Whole record</option>
        {namedFields.map(f => (
          <option key={f} value={f}>
            %{f}
          </option>
        ))}
        <option value="__custom">Custom field…</option>
      </select>

      {/* Custom field input if "Custom field…" is selected */}
      {matcher.field === '__custom' && (
        <input
          type="text"
          className="matcher-custom-field-input"
          placeholder="field name"
          onChange={e => onChange({ ...matcher, field: e.target.value || '__custom' })}
        />
      )}

      {/* Negate toggle */}
      <button
        type="button"
        className={`matcher-negate-btn${matcher.negate ? ' matcher-negate-btn--active' : ''}`}
        onClick={() => onChange({ ...matcher, negate: !matcher.negate })}
        title="Toggle negation (NOT)"
      >
        NOT
      </button>

      {/* Pattern */}
      <input
        type="text"
        className="matcher-pattern-input"
        placeholder="regex pattern"
        value={matcher.pattern}
        onChange={e => onChange({ ...matcher, pattern: e.target.value })}
      />

      <button
        type="button"
        className="matcher-remove-btn"
        onClick={onRemove}
        title="Remove matcher"
      >
        ×
      </button>
    </div>
  );
}
