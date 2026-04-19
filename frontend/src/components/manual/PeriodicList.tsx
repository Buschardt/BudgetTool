import { useState } from 'react';
import type { PeriodicTransactionSummary } from '../../types/manual';
import { deletePeriodic } from '../../api';
import { PeriodicForm } from './PeriodicForm';

interface Props {
  periodics: PeriodicTransactionSummary[];
  journalId: number;
  accounts: string[];
  onUpdated: (p: PeriodicTransactionSummary) => void;
  onDeleted: (id: number) => void;
}

export function PeriodicList({ periodics, journalId, accounts, onUpdated, onDeleted }: Props) {
  const [editingId, setEditingId] = useState<number | null>(null);
  const [deleting, setDeleting] = useState<number | null>(null);

  async function handleDelete(p: PeriodicTransactionSummary) {
    if (!confirm(`Delete budget: ${p.period}${p.description ? ` — ${p.description}` : ''}?`)) return;
    setDeleting(p.id);
    try {
      await deletePeriodic(p.id);
      onDeleted(p.id);
    } catch (err: unknown) {
      alert(err instanceof Error ? err.message : 'Delete failed');
    } finally {
      setDeleting(null);
    }
  }

  if (periodics.length === 0) {
    return <p className="manual-empty">No budgets yet.</p>;
  }

  return (
    <ul className="manual-list">
      {periodics.map(p => (
        <li key={p.id} className="manual-list-item">
          {editingId === p.id ? (
            <PeriodicForm
              editing={p}
              journalId={journalId}
              accounts={accounts}
              onSaved={updated => {
                onUpdated(updated);
                setEditingId(null);
              }}
              onCancel={() => setEditingId(null)}
            />
          ) : (
            <div className="manual-list-row">
              <span className="manual-list-main">
                <span className="manual-list-label">{p.period}</span>
                {p.description && <span className="manual-list-meta">{p.description}</span>}
                <span className="manual-list-meta">
                  {p.postings.length} posting{p.postings.length !== 1 ? 's' : ''}
                </span>
                {p.comment && <span className="manual-list-comment">{p.comment}</span>}
              </span>
              <span className="manual-list-actions">
                <button
                  type="button"
                  className="manual-btn manual-btn--edit"
                  onClick={() => setEditingId(p.id)}
                >
                  Edit
                </button>
                <button
                  type="button"
                  className="manual-btn manual-btn--delete"
                  onClick={() => handleDelete(p)}
                  disabled={deleting === p.id}
                >
                  {deleting === p.id ? '…' : 'Delete'}
                </button>
              </span>
            </div>
          )}
        </li>
      ))}
    </ul>
  );
}
