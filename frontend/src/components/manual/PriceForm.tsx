import { useState } from 'react';
import type { CommodityPriceSummary, CreatePriceRequest, UpdatePriceRequest } from '../../types/manual';
import { EMPTY_PRICE } from '../../types/manual';
import { createPrice, updatePrice } from '../../api';

interface Props {
  editing: CommodityPriceSummary | null;
  onSaved: (price: CommodityPriceSummary) => void;
  onCancel: () => void;
}

export function PriceForm({ editing, onSaved, onCancel }: Props) {
  const isNew = editing === null;
  const [form, setForm] = useState<CreatePriceRequest>(
    editing
      ? {
          date: editing.date,
          commodity: editing.commodity,
          amount: editing.amount,
          target_commodity: editing.target_commodity,
          comment: editing.comment,
        }
      : { ...EMPTY_PRICE }
  );
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');

  function patch(fields: Partial<CreatePriceRequest>) {
    setForm(prev => ({ ...prev, ...fields }));
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setSaving(true);
    setError('');
    try {
      if (isNew) {
        const result = await createPrice(form);
        onSaved(result);
      } else {
        const data: UpdatePriceRequest = form;
        const result = await updatePrice(editing.id, data);
        onSaved(result);
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Save failed');
    } finally {
      setSaving(false);
    }
  }

  return (
    <form className="manual-form" onSubmit={handleSubmit}>
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
        <div className="manual-form-field">
          <label>Commodity</label>
          <input
            type="text"
            placeholder="e.g. AAPL, EUR"
            value={form.commodity}
            onChange={e => patch({ commodity: e.target.value })}
            required
          />
        </div>
        <div className="manual-form-field">
          <label>Amount</label>
          <input
            type="text"
            inputMode="decimal"
            placeholder="e.g. 170.50"
            value={form.amount}
            onChange={e => patch({ amount: e.target.value })}
            required
          />
        </div>
        <div className="manual-form-field">
          <label>In</label>
          <input
            type="text"
            placeholder="e.g. USD"
            value={form.target_commodity}
            onChange={e => patch({ target_commodity: e.target.value })}
            required
          />
        </div>
        <div className="manual-form-field">
          <label>Comment</label>
          <input
            type="text"
            placeholder="Optional"
            value={form.comment ?? ''}
            onChange={e => patch({ comment: e.target.value })}
          />
        </div>
      </div>
      {error && <p className="manual-form-error">{error}</p>}
      <div className="manual-form-actions">
        <button type="submit" className="manual-btn manual-btn--primary" disabled={saving}>
          {saving ? 'Saving…' : isNew ? 'Add price' : 'Save'}
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
