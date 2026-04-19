import type { Posting } from '../../types/manual';

interface Props {
  postings: Posting[];
  accounts: string[];
  onChange: (postings: Posting[]) => void;
}

export function PostingRows({ postings, accounts, onChange }: Props) {
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
          {accounts.length === 0 ? (
            <select className="posting-account" disabled value="">
              <option value="">No accounts — edit journal settings</option>
            </select>
          ) : (
            <select
              className="posting-account"
              value={p.account}
              onChange={e => update(i, { account: e.target.value })}
              required
            >
              <option value="" disabled>— choose account —</option>
              {accounts.map(a => (
                <option key={a} value={a}>{a}</option>
              ))}
              {p.account && !accounts.includes(p.account) && (
                <option key={`__current_${i}`} value={p.account}>{p.account}</option>
              )}
            </select>
          )}
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
