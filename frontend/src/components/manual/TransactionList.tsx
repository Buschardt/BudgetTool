import { useState } from 'react';
import type { ManualTransactionSummary } from '../../types/manual';
import { deleteTransaction } from '../../api';
import { TransactionForm } from './TransactionForm';

interface Props {
  transactions: ManualTransactionSummary[];
  onUpdated: (txn: ManualTransactionSummary) => void;
  onDeleted: (id: number) => void;
}

export function TransactionList({ transactions, onUpdated, onDeleted }: Props) {
  const [editingId, setEditingId] = useState<number | null>(null);
  const [deleting, setDeleting] = useState<number | null>(null);

  async function handleDelete(t: ManualTransactionSummary) {
    if (!confirm(`Delete transaction: ${t.date} ${t.description}?`)) return;
    setDeleting(t.id);
    try {
      await deleteTransaction(t.id);
      onDeleted(t.id);
    } catch (err: unknown) {
      alert(err instanceof Error ? err.message : 'Delete failed');
    } finally {
      setDeleting(null);
    }
  }

  if (transactions.length === 0) {
    return <p className="manual-empty">No transactions yet.</p>;
  }

  return (
    <ul className="manual-list">
      {transactions.map(t => (
        <li key={t.id} className="manual-list-item">
          {editingId === t.id ? (
            <TransactionForm
              editing={t}
              onSaved={updated => {
                onUpdated(updated);
                setEditingId(null);
              }}
              onCancel={() => setEditingId(null)}
            />
          ) : (
            <div className="manual-list-row">
              <span className="manual-list-main">
                <span className="manual-list-date">{t.date}</span>
                {t.status && <span className="manual-list-status">{t.status}</span>}
                <span className="manual-list-label">{t.description}</span>
                <span className="manual-list-meta">
                  {t.postings.length} posting{t.postings.length !== 1 ? 's' : ''}
                </span>
                {t.comment && <span className="manual-list-comment">{t.comment}</span>}
              </span>
              <span className="manual-list-actions">
                <button
                  type="button"
                  className="manual-btn manual-btn--edit"
                  onClick={() => setEditingId(t.id)}
                >
                  Edit
                </button>
                <button
                  type="button"
                  className="manual-btn manual-btn--delete"
                  onClick={() => handleDelete(t)}
                  disabled={deleting === t.id}
                >
                  {deleting === t.id ? '…' : 'Delete'}
                </button>
              </span>
            </div>
          )}
        </li>
      ))}
    </ul>
  );
}
