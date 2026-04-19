import { useState } from 'react';
import type { CommodityPriceSummary } from '../../types/manual';
import { deletePrice } from '../../api';
import { PriceForm } from './PriceForm';

interface Props {
  prices: CommodityPriceSummary[];
  journalId: number;
  onUpdated: (price: CommodityPriceSummary) => void;
  onDeleted: (id: number) => void;
}

export function PriceList({ prices, journalId, onUpdated, onDeleted }: Props) {
  const [editingId, setEditingId] = useState<number | null>(null);
  const [deleting, setDeleting] = useState<number | null>(null);

  async function handleDelete(p: CommodityPriceSummary) {
    if (!confirm(`Delete price: ${p.date} ${p.commodity} ${p.amount} ${p.target_commodity}?`)) return;
    setDeleting(p.id);
    try {
      await deletePrice(p.id);
      onDeleted(p.id);
    } catch (err: unknown) {
      alert(err instanceof Error ? err.message : 'Delete failed');
    } finally {
      setDeleting(null);
    }
  }

  if (prices.length === 0) {
    return <p className="manual-empty">No prices yet.</p>;
  }

  return (
    <ul className="manual-list">
      {prices.map(p => (
        <li key={p.id} className="manual-list-item">
          {editingId === p.id ? (
            <PriceForm
              editing={p}
              journalId={journalId}
              onSaved={updated => {
                onUpdated(updated);
                setEditingId(null);
              }}
              onCancel={() => setEditingId(null)}
            />
          ) : (
            <div className="manual-list-row">
              <span className="manual-list-main">
                <span className="manual-list-date">{p.date}</span>
                <span className="manual-list-label">{p.commodity}</span>
                <span className="manual-list-amount">
                  {p.amount} {p.target_commodity}
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
