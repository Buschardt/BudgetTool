import type { ConditionalRule, MatchGroup, Matcher, FieldAssignment } from '../../types/rules';
import { MatcherRow } from './MatcherRow';
import { AssignmentRow } from './AssignmentRow';
import { ConditionalTableEditor } from './ConditionalTableEditor';
import './ConditionalRuleCard.css';

interface Props {
  rule: ConditionalRule;
  index: number;
  total: number;
  csvFields: string[];
  onChange: (r: ConditionalRule) => void;
  onRemove: () => void;
  onMoveUp: () => void;
  onMoveDown: () => void;
}

function emptyMatcher(): Matcher {
  return { pattern: '', negate: false };
}

function emptyGroup(): MatchGroup {
  return { matchers: [emptyMatcher()] };
}

function emptyAssignment(): FieldAssignment {
  return { field: 'account2', value: '' };
}

export function ConditionalRuleCard({
  rule,
  index,
  total,
  csvFields,
  onChange,
  onRemove,
  onMoveUp,
  onMoveDown,
}: Props) {
  const groups = rule.matchGroups ?? [];
  const assignments = rule.assignments ?? [];

  // --- Matcher group helpers ---
  function updateGroup(gi: number, group: MatchGroup) {
    const next = [...groups];
    next[gi] = group;
    onChange({ ...rule, matchGroups: next });
  }

  function addGroup() {
    onChange({ ...rule, matchGroups: [...groups, emptyGroup()] });
  }

  function removeGroup(gi: number) {
    onChange({ ...rule, matchGroups: groups.filter((_, i) => i !== gi) });
  }

  function updateMatcher(gi: number, mi: number, m: Matcher) {
    const newMatchers = [...groups[gi].matchers];
    newMatchers[mi] = m;
    updateGroup(gi, { ...groups[gi], matchers: newMatchers });
  }

  function addMatcher(gi: number) {
    updateGroup(gi, { ...groups[gi], matchers: [...groups[gi].matchers, emptyMatcher()] });
  }

  function removeMatcher(gi: number, mi: number) {
    const newMatchers = groups[gi].matchers.filter((_, i) => i !== mi);
    if (newMatchers.length === 0) {
      // Remove the entire group if it's empty
      removeGroup(gi);
    } else {
      updateGroup(gi, { ...groups[gi], matchers: newMatchers });
    }
  }

  // --- Assignment helpers ---
  function updateAssignment(i: number, a: FieldAssignment) {
    const next = [...assignments];
    next[i] = a;
    onChange({ ...rule, assignments: next });
  }

  function addAssignment() {
    onChange({ ...rule, assignments: [...assignments, emptyAssignment()] });
  }

  function removeAssignment(i: number) {
    onChange({ ...rule, assignments: assignments.filter((_, ai) => ai !== i) });
  }

  const isTable = rule.type === 'table';

  return (
    <div className="crc">
      {/* Card header */}
      <div className="crc-header">
        <span className="crc-index">Rule {index + 1}</span>

        <div className="crc-type-toggle">
          <button
            type="button"
            className={`crc-type-btn${!isTable ? ' crc-type-btn--active' : ''}`}
            onClick={() => onChange({ ...rule, type: 'block' })}
          >
            Conditions
          </button>
          <button
            type="button"
            className={`crc-type-btn${isTable ? ' crc-type-btn--active' : ''}`}
            onClick={() => onChange({ ...rule, type: 'table' })}
          >
            Table
          </button>
        </div>

        <div className="crc-header-actions">
          <button
            type="button"
            className="crc-move-btn"
            onClick={onMoveUp}
            disabled={index === 0}
            title="Move up"
          >
            ↑
          </button>
          <button
            type="button"
            className="crc-move-btn"
            onClick={onMoveDown}
            disabled={index === total - 1}
            title="Move down"
          >
            ↓
          </button>
          <button
            type="button"
            className="crc-remove-btn"
            onClick={onRemove}
            title="Remove rule"
          >
            Remove
          </button>
        </div>
      </div>

      {isTable ? (
        <ConditionalTableEditor rule={rule} onChange={onChange} />
      ) : (
        <div className="crc-body">
          {/* WHEN section */}
          <div className="crc-when">
            <div className="crc-section-label">When</div>
            <div className="crc-groups">
              {groups.map((group, gi) => (
                <div key={gi} className="crc-group">
                  {gi > 0 && <div className="crc-or-divider">OR</div>}
                  <div className="crc-group-matchers">
                    {group.matchers.map((m, mi) => (
                      <MatcherRow
                        key={mi}
                        matcher={m}
                        onChange={updated => updateMatcher(gi, mi, updated)}
                        onRemove={() => removeMatcher(gi, mi)}
                        csvFields={csvFields}
                        isFirst={mi === 0}
                      />
                    ))}
                  </div>
                  <div className="crc-group-actions">
                    <button
                      type="button"
                      className="crc-add-matcher-btn"
                      onClick={() => addMatcher(gi)}
                    >
                      + AND condition
                    </button>
                    {groups.length > 1 && (
                      <button
                        type="button"
                        className="crc-remove-group-btn"
                        onClick={() => removeGroup(gi)}
                      >
                        Remove group
                      </button>
                    )}
                  </div>
                </div>
              ))}

              <button type="button" className="crc-add-group-btn" onClick={addGroup}>
                + OR group
              </button>
            </div>
          </div>

          {/* THEN section */}
          <div className="crc-then">
            <div className="crc-section-label">Then</div>
            <div className="crc-assignments">
              {assignments.map((a, i) => (
                <AssignmentRow
                  key={i}
                  assignment={a}
                  onChange={v => updateAssignment(i, v)}
                  onRemove={() => removeAssignment(i)}
                  csvFields={csvFields}
                />
              ))}
              <button type="button" className="crc-add-assignment-btn" onClick={addAssignment}>
                + Set field
              </button>
            </div>

            <div className="crc-flags">
              <label className="crc-flag-label">
                <input
                  type="checkbox"
                  checked={rule.skip ?? false}
                  onChange={e => onChange({ ...rule, skip: e.target.checked || undefined })}
                />
                Skip this transaction (don't import)
              </label>
              <label className="crc-flag-label">
                <input
                  type="checkbox"
                  checked={rule.end ?? false}
                  onChange={e => onChange({ ...rule, end: e.target.checked || undefined })}
                />
                End (skip all remaining transactions in file)
              </label>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
