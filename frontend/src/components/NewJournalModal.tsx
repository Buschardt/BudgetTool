import { useState } from 'react';
import { createJournal } from '../api';
import type { FileInfo } from '../api';
import { EMPTY_JOURNAL_SETTINGS } from '../types/journal';
import './NewJournalModal.css';

interface Props {
  onCreated: (file: FileInfo) => void;
  onCancel: () => void;
}

type State = 'idle' | 'creating' | 'error';

export function NewJournalModal({ onCreated, onCancel }: Props) {
  const [name, setName] = useState('');
  const [state, setState] = useState<State>('idle');
  const [error, setError] = useState('');

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const trimmed = name.trim();
    if (!trimmed) return;
    setState('creating');
    setError('');
    try {
      const file = await createJournal(trimmed, EMPTY_JOURNAL_SETTINGS);
      onCreated(file);
    } catch (err) {
      setState('error');
      setError(err instanceof Error ? err.message : 'Failed to create journal');
    }
  }

  return (
    <div className="new-journal-modal-overlay" onClick={onCancel}>
      <div className="new-journal-modal" onClick={e => e.stopPropagation()}>
        <div className="new-journal-modal-header">
          <h2 className="new-journal-modal-title">New journal</h2>
          <button
            type="button"
            className="new-journal-modal-close"
            onClick={onCancel}
          >
            ×
          </button>
        </div>

        <form className="new-journal-modal-body" onSubmit={handleSubmit}>
          <label className="new-journal-modal-label" htmlFor="journal-name">
            Name
          </label>
          <input
            id="journal-name"
            type="text"
            className="new-journal-modal-input"
            placeholder="e.g. 2026"
            value={name}
            onChange={e => {
              setName(e.target.value);
              setState('idle');
              setError('');
            }}
            autoFocus
            disabled={state === 'creating'}
          />
          <p className="new-journal-modal-hint">
            .journal extension is added automatically if omitted.
          </p>

          {state === 'error' && (
            <p className="new-journal-modal-error">{error}</p>
          )}

          <div className="new-journal-modal-footer">
            <button
              type="submit"
              className="new-journal-modal-create-btn"
              disabled={!name.trim() || state === 'creating'}
            >
              {state === 'creating' ? 'Creating…' : 'Create'}
            </button>
            <button
              type="button"
              className="new-journal-modal-cancel-btn"
              onClick={onCancel}
              disabled={state === 'creating'}
            >
              Cancel
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
