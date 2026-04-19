import { useEffect, useMemo, useState } from 'react';
import {
  createJournal,
  getJournalSettings,
  updateJournalSettings,
} from '../api';
import type {
  FileInfo,
  JournalSettings,
} from '../api';
import { EMPTY_JOURNAL_SETTINGS } from '../types/journal';
import './JournalSettingsModal.css';

interface Props {
  mode: 'create' | 'edit';
  file?: FileInfo | null;
  allFiles: FileInfo[];
  onSaved: (file: FileInfo) => void;
  onCancel: () => void;
}

const ACCOUNT_ROOT_OPTIONS = [
  'assets',
  'liabilities',
  'equity',
  'income',
  'expenses',
] as const;

type AccountRoot = (typeof ACCOUNT_ROOT_OPTIONS)[number];

interface EditableAccountRow {
  root: string;
  segments: string[];
}

function cloneSettings(settings: JournalSettings): JournalSettings {
  return {
    default_commodity: settings.default_commodity ?? '',
    decimal_mark: settings.decimal_mark ?? null,
    commodities: [...settings.commodities],
    accounts: [...settings.accounts],
    includes: [...settings.includes],
  };
}

function isAccountRoot(value: string): value is AccountRoot {
  return ACCOUNT_ROOT_OPTIONS.includes(value as AccountRoot);
}

function canonicalizeAccountRoot(root: string): string {
  const normalized = root.trim().toLowerCase();
  return isAccountRoot(normalized) ? normalized : root.trim();
}

function createEmptyAccountRow(): EditableAccountRow {
  return {
    root: 'assets',
    segments: [''],
  };
}

function parseAccountName(name: string): EditableAccountRow {
  const trimmed = name.trim();
  if (!trimmed) {
    return { root: '', segments: [''] };
  }

  const parts = trimmed.split(':');
  return {
    root: canonicalizeAccountRoot(parts[0] ?? ''),
    segments: parts.length > 1 ? parts.slice(1).map(segment => segment.trim()) : [],
  };
}

function serializeAccountRow(row: EditableAccountRow): string {
  const root = canonicalizeAccountRoot(row.root);
  const segments = row.segments.map(segment => segment.trim());
  return [root, ...segments].join(':').trim();
}

function getAccountRowError(row: EditableAccountRow): string | null {
  const root = canonicalizeAccountRoot(row.root);
  if (!isAccountRoot(root)) {
    return 'Choose one of the canonical account roots.';
  }

  for (const segment of row.segments) {
    if (segment.includes(':')) {
      return 'Each category must be in its own field.';
    }
    if (!segment.trim()) {
      return 'Category fields cannot be blank.';
    }
  }

  return null;
}

export function JournalSettingsModal({
  mode,
  file,
  allFiles,
  onSaved,
  onCancel,
}: Props) {
  const [name, setName] = useState(mode === 'edit' ? (file?.filename ?? '') : '');
  const [settings, setSettings] = useState<JournalSettings>(cloneSettings(EMPTY_JOURNAL_SETTINGS));
  const [accountRows, setAccountRows] = useState<EditableAccountRow[]>([]);
  const [loading, setLoading] = useState(mode === 'edit');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');

  const journalOptions = useMemo(
    () =>
      allFiles.filter(
        candidate => candidate.file_type === 'journal' && candidate.id !== file?.id
      ),
    [allFiles, file?.id]
  );

  useEffect(() => {
    if (mode !== 'edit' || !file) return;
    setLoading(true);
    setError('');
    getJournalSettings(file.id)
      .then(detail => {
        setName(detail.file.filename);
        setSettings(cloneSettings(detail.settings));
        setAccountRows(detail.settings.accounts.map(entry => parseAccountName(entry.name)));
      })
      .catch(err => {
        setError(err instanceof Error ? err.message : 'Failed to load journal settings');
      })
      .finally(() => setLoading(false));
  }, [mode, file]);

  const accountState = useMemo(() => {
    const serializedNames = accountRows.map(serializeAccountRow);
    const counts = new Map<string, number>();

    for (const name of serializedNames) {
      if (!name) continue;
      counts.set(name, (counts.get(name) ?? 0) + 1);
    }

    return accountRows.map((row, index) => {
      const serialized = serializedNames[index];
      const shapeError = getAccountRowError(row);
      const error =
        shapeError ??
        (serialized && (counts.get(serialized) ?? 0) > 1
          ? `Duplicate account declaration: ${serialized}`
          : null);

      return {
        serialized,
        error,
        hasInvalidRoot: !isAccountRoot(canonicalizeAccountRoot(row.root)),
      };
    });
  }, [accountRows]);

  const hasAccountErrors = accountState.some(entry => entry.error);

  function patch(patch: Partial<JournalSettings>) {
    setSettings(prev => ({ ...prev, ...patch }));
  }

  function updateCommodity(index: number, sample: string) {
    patch({
      commodities: settings.commodities.map((entry, i) =>
        i === index ? { sample } : entry
      ),
    });
  }

  function updateAccountRoot(index: number, root: string) {
    setAccountRows(prev =>
      prev.map((entry, i) => (i === index ? { ...entry, root } : entry))
    );
  }

  function updateAccountSegment(index: number, segmentIndex: number, value: string) {
    setAccountRows(prev =>
      prev.map((entry, i) =>
        i === index
          ? {
              ...entry,
              segments: entry.segments.map((segment, currentIndex) =>
                currentIndex === segmentIndex ? value : segment
              ),
            }
          : entry
      )
    );
  }

  function addAccountRow() {
    setAccountRows(prev => [...prev, createEmptyAccountRow()]);
  }

  function removeAccountRow(index: number) {
    setAccountRows(prev => prev.filter((_, i) => i !== index));
  }

  function addAccountSegment(index: number) {
    setAccountRows(prev =>
      prev.map((entry, i) =>
        i === index ? { ...entry, segments: [...entry.segments, ''] } : entry
      )
    );
  }

  function removeAccountSegment(index: number, segmentIndex: number) {
    setAccountRows(prev =>
      prev.map((entry, i) =>
        i === index
          ? {
              ...entry,
              segments: entry.segments.filter((_, currentIndex) => currentIndex !== segmentIndex),
            }
          : entry
      )
    );
  }

  function toggleInclude(id: number) {
    patch({
      includes: settings.includes.includes(id)
        ? settings.includes.filter(existing => existing !== id)
        : [...settings.includes, id],
    });
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (hasAccountErrors) {
      setError('Fix invalid account declarations before saving.');
      return;
    }

    const payload: JournalSettings = {
      ...settings,
      accounts: accountState
        .map(entry => ({ name: entry.serialized }))
        .filter(entry => entry.name.trim().length > 0),
    };

    setSaving(true);
    setError('');
    try {
      if (mode === 'create') {
        const created = await createJournal(name.trim(), payload);
        onSaved(created);
      } else if (file) {
        const updated = await updateJournalSettings(file.id, payload);
        onSaved(updated.file);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save journal settings');
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="journal-settings-modal-overlay" onClick={onCancel}>
      <div className="journal-settings-modal" onClick={e => e.stopPropagation()}>
        <div className="journal-settings-modal-header">
          <h2 className="journal-settings-modal-title">
            {mode === 'create' ? 'New journal' : 'Journal settings'}
          </h2>
          <button
            type="button"
            className="journal-settings-modal-close"
            onClick={onCancel}
          >
            ×
          </button>
        </div>

        <form className="journal-settings-modal-body" onSubmit={handleSubmit}>
          {mode === 'create' ? (
            <div className="journal-settings-section">
              <label className="journal-settings-label" htmlFor="journal-name">
                Name
              </label>
              <input
                id="journal-name"
                type="text"
                className="journal-settings-input"
                placeholder="e.g. 2026"
                value={name}
                onChange={e => setName(e.target.value)}
                disabled={saving}
                autoFocus
              />
              <p className="journal-settings-hint">
                `.journal` extension is added automatically if omitted.
              </p>
            </div>
          ) : (
            <div className="journal-settings-section">
              <label className="journal-settings-label">Journal</label>
              <div className="journal-settings-readonly">{name}</div>
            </div>
          )}

          {loading ? (
            <p className="journal-settings-loading">Loading…</p>
          ) : (
            <>
              <div className="journal-settings-grid">
                <div className="journal-settings-section">
                  <label className="journal-settings-label" htmlFor="default-commodity">
                    Default commodity
                  </label>
                  <input
                    id="default-commodity"
                    type="text"
                    className="journal-settings-input"
                    placeholder="e.g. USD or $"
                    value={settings.default_commodity ?? ''}
                    onChange={e => patch({ default_commodity: e.target.value })}
                    disabled={saving}
                  />
                </div>

                <div className="journal-settings-section">
                  <label className="journal-settings-label" htmlFor="decimal-mark">
                    Decimal separator
                  </label>
                  <select
                    id="decimal-mark"
                    className="journal-settings-input"
                    value={settings.decimal_mark ?? ''}
                    onChange={e =>
                      patch({
                        decimal_mark:
                          e.target.value === '' ? null : (e.target.value as '.' | ','),
                      })
                    }
                    disabled={saving}
                  >
                    <option value="">Use hledger default</option>
                    <option value=".">.</option>
                    <option value=",">,</option>
                  </select>
                </div>
              </div>

              <div className="journal-settings-section">
                <div className="journal-settings-section-header">
                  <h3>Commodity formatting</h3>
                  <button
                    type="button"
                    className="journal-settings-add-btn"
                    onClick={() =>
                      patch({
                        commodities: [...settings.commodities, { sample: '' }],
                      })
                    }
                    disabled={saving}
                  >
                    + Add commodity
                  </button>
                </div>
                <p className="journal-settings-hint">
                  Enter full `commodity` samples like `$1,000.00` or `1.000,00 EUR`.
                </p>
                <div className="journal-settings-list">
                  {settings.commodities.map((entry, index) => (
                    <div key={`commodity-${index}`} className="journal-settings-row">
                      <input
                        type="text"
                        className="journal-settings-input"
                        value={entry.sample}
                        onChange={e => updateCommodity(index, e.target.value)}
                        disabled={saving}
                      />
                      <button
                        type="button"
                        className="journal-settings-remove-btn"
                        onClick={() =>
                          patch({
                            commodities: settings.commodities.filter((_, i) => i !== index),
                          })
                        }
                        disabled={saving}
                      >
                        Remove
                      </button>
                    </div>
                  ))}
                  {settings.commodities.length === 0 && (
                    <p className="journal-settings-empty">No commodity directives yet.</p>
                  )}
                </div>
              </div>

              <div className="journal-settings-section">
                <div className="journal-settings-section-header">
                  <h3>Account declarations</h3>
                  <button
                    type="button"
                    className="journal-settings-add-btn"
                    onClick={addAccountRow}
                    disabled={saving}
                  >
                    + Add account
                  </button>
                </div>
                <p className="journal-settings-hint">
                  Pick the top-level account type, then add one field for each subaccount
                  category.
                </p>
                <div className="journal-settings-list">
                  {accountRows.map((entry, index) => (
                    <div key={`account-${index}`} className="journal-settings-account-card">
                      <div className="journal-settings-account-path">
                        <select
                          className={`journal-settings-input journal-settings-account-root${
                            accountState[index]?.hasInvalidRoot
                              ? ' journal-settings-input--invalid'
                              : ''
                          }`}
                          value={entry.root}
                          onChange={e => updateAccountRoot(index, e.target.value)}
                          disabled={saving}
                        >
                          {accountState[index]?.hasInvalidRoot && (
                            <option value={entry.root}>
                              Unsupported root: {entry.root || '(empty)'}
                            </option>
                          )}
                          {ACCOUNT_ROOT_OPTIONS.map(option => (
                            <option key={option} value={option}>
                              {option}
                            </option>
                          ))}
                        </select>

                        {entry.segments.map((segment, segmentIndex) => (
                          <div
                            key={`account-${index}-segment-${segmentIndex}`}
                            className="journal-settings-account-segment-group"
                          >
                            <span className="journal-settings-account-separator">:</span>
                            <div className="journal-settings-account-segment">
                              <input
                                type="text"
                                className={`journal-settings-input${
                                  accountState[index]?.error ? ' journal-settings-input--invalid' : ''
                                }`}
                                value={segment}
                                onChange={e =>
                                  updateAccountSegment(index, segmentIndex, e.target.value)
                                }
                                disabled={saving}
                                placeholder={
                                  segmentIndex === 0 ? 'e.g. bank1' : 'e.g. savings'
                                }
                              />
                              <button
                                type="button"
                                className="journal-settings-remove-btn"
                                onClick={() => removeAccountSegment(index, segmentIndex)}
                                disabled={saving}
                              >
                                Remove category
                              </button>
                            </div>
                          </div>
                        ))}
                      </div>

                      <div className="journal-settings-account-actions">
                        <button
                          type="button"
                          className="journal-settings-add-btn"
                          onClick={() => addAccountSegment(index)}
                          disabled={saving}
                        >
                          + Add category
                        </button>
                        <button
                          type="button"
                          className="journal-settings-remove-btn"
                          onClick={() => removeAccountRow(index)}
                          disabled={saving}
                        >
                          Remove account
                        </button>
                      </div>

                      {accountState[index]?.error && (
                        <p className="journal-settings-account-error">
                          {accountState[index].error}
                        </p>
                      )}
                    </div>
                  ))}
                  {accountRows.length === 0 && (
                    <p className="journal-settings-empty">No declared accounts yet.</p>
                  )}
                </div>
              </div>

              <div className="journal-settings-section">
                <div className="journal-settings-section-header">
                  <h3>Included journal files</h3>
                </div>
                <div className="journal-settings-includes">
                  {journalOptions.map(option => (
                    <label key={option.id} className="journal-settings-include-item">
                      <input
                        type="checkbox"
                        checked={settings.includes.includes(option.id)}
                        onChange={() => toggleInclude(option.id)}
                        disabled={saving}
                      />
                      <span>{option.filename}</span>
                    </label>
                  ))}
                  {journalOptions.length === 0 && (
                    <p className="journal-settings-empty">No other journal files available.</p>
                  )}
                </div>
              </div>
            </>
          )}

          {error && <p className="journal-settings-error">{error}</p>}

          <div className="journal-settings-modal-footer">
            <button
              type="submit"
              className="journal-settings-save-btn"
              disabled={(mode === 'create' && !name.trim()) || saving || loading || hasAccountErrors}
            >
              {saving ? 'Saving…' : mode === 'create' ? 'Create' : 'Save'}
            </button>
            <button
              type="button"
              className="journal-settings-cancel-btn"
              onClick={onCancel}
              disabled={saving}
            >
              Cancel
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
