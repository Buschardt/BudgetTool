import { useState } from 'react';
import type { ReportParams } from '../api';
import { DateRangePicker } from './DateRangePicker';
import './ReportFilters.css';

interface ReportFiltersProps {
  onChange: (params: ReportParams) => void;
  showDepth?: boolean;
  showAccount?: boolean;
}

export function ReportFilters({ onChange, showDepth = true, showAccount = true }: ReportFiltersProps) {
  const [begin, setBegin] = useState('');
  const [end, setEnd] = useState('');
  const [depth, setDepth] = useState<number | undefined>(undefined);
  const [account, setAccount] = useState('');

  function emit(updates: Partial<{ begin: string; end: string; depth: number | undefined; account: string }>) {
    const next = {
      begin: updates.begin !== undefined ? updates.begin : begin,
      end: updates.end !== undefined ? updates.end : end,
      depth: updates.depth !== undefined ? updates.depth : depth,
      account: updates.account !== undefined ? updates.account : account,
    };
    const params: ReportParams = {};
    if (next.begin) params.begin = next.begin;
    if (next.end) params.end = next.end;
    if (next.depth !== undefined) params.depth = next.depth;
    if (next.account) params.account = next.account;
    onChange(params);
  }

  return (
    <div className="report-filters">
      <DateRangePicker
        begin={begin}
        end={end}
        onChange={(b, e) => {
          setBegin(b);
          setEnd(e);
          emit({ begin: b, end: e });
        }}
      />

      {showDepth && (
        <label className="report-filters-field">
          <span className="report-filters-label">Depth</span>
          <input
            type="number"
            className="report-filters-input report-filters-input--narrow"
            min={1}
            max={9}
            placeholder="all"
            value={depth ?? ''}
            onChange={e => {
              const v = e.target.value ? Number(e.target.value) : undefined;
              setDepth(v);
              emit({ depth: v });
            }}
          />
        </label>
      )}

      {showAccount && (
        <label className="report-filters-field">
          <span className="report-filters-label">Account</span>
          <input
            type="text"
            className="report-filters-input"
            placeholder="e.g. expenses"
            value={account}
            onChange={e => {
              setAccount(e.target.value);
              emit({ account: e.target.value });
            }}
          />
        </label>
      )}
    </div>
  );
}
