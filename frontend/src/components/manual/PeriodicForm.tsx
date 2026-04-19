import { useState } from 'react';
import type {
  CreatePeriodicRequest,
  PeriodicTransactionSummary,
  UpdatePeriodicRequest,
} from '../../types/manual';
import { EMPTY_PERIODIC } from '../../types/manual';
import { createPeriodic, updatePeriodic } from '../../api';
import { PostingRows } from './PostingRows';

interface Props {
  editing: PeriodicTransactionSummary | null;
  onSaved: (p: PeriodicTransactionSummary) => void;
  onCancel: () => void;
}

export function PeriodicForm({ editing, onSaved, onCancel }: Props) {
  const isNew = editing === null;
  const [form, setForm] = useState<CreatePeriodicRequest>(
    editing
      ? {
          period: editing.period,
          description: editing.description,
          comment: editing.comment,
          postings: editing.postings,
        }
      : { ...EMPTY_PERIODIC, postings: [...EMPTY_PERIODIC.postings] }
  );
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');

  function patch(fields: Partial<CreatePeriodicRequest>) {
    setForm(prev => ({ ...prev, ...fields }));
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setSaving(true);
    setError('');
    try {
      if (isNew) {
        const result = await createPeriodic(form);
        onSaved(result);
        setForm({ ...EMPTY_PERIODIC, postings: [...EMPTY_PERIODIC.postings] });
      } else {
        const data: UpdatePeriodicRequest = form;
        const result = await updatePeriodic(editing.id, data);
        onSaved(result);
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Save failed');
    } finally {
      setSaving(false);
    }
  }

  return (
    <form className="manual-form manual-form--wide" onSubmit={handleSubmit}>
      <div className="manual-form-row">
        <div className="manual-form-field manual-form-field--grow">
          <label>Period</label>
          <input
            type="text"
            placeholder="e.g. monthly, every 2 weeks from 2026-01-01"
            value={form.period}
            onChange={e => patch({ period: e.target.value })}
            required
          />
        </div>
        <div className="manual-form-field manual-form-field--grow">
          <label>Description</label>
          <input
            type="text"
            placeholder="e.g. Groceries budget"
            value={form.description ?? ''}
            onChange={e => patch({ description: e.target.value })}
          />
        </div>
      </div>
      <div className="manual-form-row">
        <div className="manual-form-field manual-form-field--full">
          <label>Comment</label>
          <input
            type="text"
            placeholder="Optional"
            value={form.comment ?? ''}
            onChange={e => patch({ comment: e.target.value })}
          />
        </div>
      </div>
      <div className="manual-form-field">
        <label>Postings</label>
        <PostingRows postings={form.postings} onChange={postings => patch({ postings })} />
      </div>
      {error && <p className="manual-form-error">{error}</p>}
      <div className="manual-form-actions">
        <button type="submit" className="manual-btn manual-btn--primary" disabled={saving}>
          {saving ? 'Saving…' : isNew ? 'Add budget' : 'Save'}
        </button>
        {!isNew && (
          <button type="button" className="manual-btn manual-btn--cancel" onClick={onCancel}>
            Cancel
          </button>
        )}
      </div>
    </form>
  );
}
