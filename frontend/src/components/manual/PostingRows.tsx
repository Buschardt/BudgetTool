import type { Posting } from '../../types/manual';

interface Props {
  postings: Posting[];
  onChange: (postings: Posting[]) => void;
}

export function PostingRows({ postings, onChange }: Props) {
  function update(index: number, patch: Partial<Posting>) {
    const next = postings.map((p, i) => (i === index ? { ...p, ...patch } : p));
    onChange(next);
  }

  function add() {
    onChange([...postings, { account: '' }]);
  }

  function remove(index: number) {
    onChange(postings.filter((_, i) => i !== index));
  }

  return (
    <div className="posting-rows">
      {postings.map((p, i) => (
        <div key={i} className="posting-row">
          <input
            type="text"
            placeholder="Account (e.g. expenses:groceries)"
            value={p.account}
            onChange={e => update(i, { account: e.target.value })}
            className="posting-account"
          />
          <input
            type="text"
            inputMode="decimal"
            placeholder="Amount (optional)"
            value={p.amount ?? ''}
            onChange={e => update(i, { amount: e.target.value })}
            className="posting-amount"
          />
          <input
            type="text"
            placeholder="Commodity (e.g. USD)"
            value={p.commodity ?? ''}
            onChange={e => update(i, { commodity: e.target.value })}
            className="posting-commodity"
          />
          {postings.length > 1 && (
            <button type="button" className="posting-remove" onClick={() => remove(i)}>
              ×
            </button>
          )}
        </div>
      ))}
      <button type="button" className="posting-add" onClick={add}>
        + Add posting
      </button>
    </div>
  );
}
