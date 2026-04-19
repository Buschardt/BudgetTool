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

function cloneSettings(settings: JournalSettings): JournalSettings {
  return {
    default_commodity: settings.default_commodity ?? '',
    decimal_mark: settings.decimal_mark ?? null,
    commodities: [...settings.commodities],
    accounts: [...settings.accounts],
    includes: [...settings.includes],
  };
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
      })
      .catch(err => {
        setError(err instanceof Error ? err.message : 'Failed to load journal settings');
      })
      .finally(() => setLoading(false));
  }, [mode, file]);

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

  function updateAccount(index: number, name: string) {
    patch({
      accounts: settings.accounts.map((entry, i) =>
        i === index ? { name } : entry
      ),
    });
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
    setSaving(true);
    setError('');
    try {
      if (mode === 'create') {
        const created = await createJournal(name.trim(), settings);
        onSaved(created);
      } else if (file) {
        const updated = await updateJournalSettings(file.id, settings);
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
                    onClick={() =>
                      patch({
                        accounts: [...settings.accounts, { name: '' }],
                      })
                    }
                    disabled={saving}
                  >
                    + Add account
                  </button>
                </div>
                <div className="journal-settings-list">
                  {settings.accounts.map((entry, index) => (
                    <div key={`account-${index}`} className="journal-settings-row">
                      <input
                        type="text"
                        className="journal-settings-input"
                        value={entry.name}
                        onChange={e => updateAccount(index, e.target.value)}
                        disabled={saving}
                        placeholder="e.g. assets:bank:checking"
                      />
                      <button
                        type="button"
                        className="journal-settings-remove-btn"
                        onClick={() =>
                          patch({
                            accounts: settings.accounts.filter((_, i) => i !== index),
                          })
                        }
                        disabled={saving}
                      >
                        Remove
                      </button>
                    </div>
                  ))}
                  {settings.accounts.length === 0 && (
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
              disabled={(mode === 'create' && !name.trim()) || saving || loading}
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
