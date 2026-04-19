import { useState } from 'react';
import type {
  CreateTransactionRequest,
  ManualTransactionSummary,
  UpdateTransactionRequest,
} from '../../types/manual';
import { EMPTY_TRANSACTION } from '../../types/manual';
import { createTransaction, updateTransaction } from '../../api';
import { PostingRows } from './PostingRows';

interface Props {
  editing: ManualTransactionSummary | null;
  journalId: number;
  accounts: string[];
  onSaved: (txn: ManualTransactionSummary) => void;
  onCancel: () => void;
}

export function TransactionForm({ editing, journalId, accounts, onSaved, onCancel }: Props) {
  const isNew = editing === null;
  const [form, setForm] = useState<CreateTransactionRequest>(
    editing
      ? {
          journal_file_id: editing.journal_file_id,
          date: editing.date,
          status: editing.status,
          code: editing.code,
          description: editing.description,
          comment: editing.comment,
          postings: editing.postings,
        }
      : { ...EMPTY_TRANSACTION, journal_file_id: journalId, postings: [...EMPTY_TRANSACTION.postings] }
  );
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');

  function patch(fields: Partial<CreateTransactionRequest>) {
    setForm(prev => ({ ...prev, ...fields }));
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setSaving(true);
    setError('');
    try {
      if (isNew) {
        const result = await createTransaction({ ...form, journal_file_id: journalId });
        onSaved(result);
        setForm({ ...EMPTY_TRANSACTION, journal_file_id: journalId, postings: [...EMPTY_TRANSACTION.postings] });
      } else {
        const data: UpdateTransactionRequest = form;
        const result = await updateTransaction(editing.id, data);
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
        <div className="manual-form-field">
          <label>Date</label>
          <input
            type="date"
            value={form.date}
            onChange={e => patch({ date: e.target.value })}
            required
          />
        </div>
        <div className="manual-form-field manual-form-field--narrow">
          <label>Status</label>
          <select value={form.status ?? ''} onChange={e => patch({ status: e.target.value })}>
            <option value="">—</option>
            <option value="*">* Cleared</option>
            <option value="!">! Pending</option>
          </select>
        </div>
        <div className="manual-form-field manual-form-field--narrow">
          <label>Code</label>
          <input
            type="text"
            placeholder="Optional"
            value={form.code ?? ''}
            onChange={e => patch({ code: e.target.value })}
          />
        </div>
        <div className="manual-form-field manual-form-field--grow">
          <label>Description</label>
          <input
            type="text"
            placeholder="e.g. Groceries"
            value={form.description}
            onChange={e => patch({ description: e.target.value })}
            required
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
        <PostingRows
          postings={form.postings}
          accounts={accounts}
          onChange={postings => patch({ postings })}
        />
      </div>
      {error && <p className="manual-form-error">{error}</p>}
      <div className="manual-form-actions">
        <button type="submit" className="manual-btn manual-btn--primary" disabled={saving}>
          {saving ? 'Saving…' : isNew ? 'Add transaction' : 'Save'}
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
