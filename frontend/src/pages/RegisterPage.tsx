import { useState } from 'react';
import { getRegister } from '../api';
import type { ReportParams } from '../api';
import { useReport } from '../hooks/useReport';
import { parseRegister } from '../lib/hledger-parse';
import { formatCurrency, formatDate } from '../lib/format';
import { ReportFilters } from '../components/ReportFilters';
import { DataTable } from '../components/DataTable';
import type { Column } from '../components/DataTable';
import type { RegisterEntry } from '../types/hledger';
import './RegisterPage.css';

const COLUMNS: Column<RegisterEntry>[] = [
  { key: 'date', header: 'Date', render: r => formatDate(r.date) },
  { key: 'description', header: 'Description' },
  { key: 'account', header: 'Account' },
  {
    key: 'amount',
    header: 'Amount',
    align: 'right',
    render: r => (
      <span style={{ color: r.amount >= 0 ? 'var(--success)' : 'var(--danger)' }}>
        {formatCurrency(r.amount, r.commodity)}
      </span>
    ),
  },
  {
    key: 'runningTotal',
    header: 'Running Total',
    align: 'right',
    render: r => formatCurrency(r.runningTotal, r.commodity),
  },
];

export function RegisterPage() {
  const [params, setParams] = useState<ReportParams>({});
  const { data: raw, loading, error, refetch } = useReport(getRegister, params);

  const entries = raw ? parseRegister(raw) : [];

  return (
    <div className="register-page">
      <h1>Register</h1>
      <ReportFilters onChange={setParams} showDepth={false} showAccount />

      {loading && <div className="report-loading">Loading…</div>}
      {error && (
        <div className="report-error">
          {error}
          {error.includes('no journal') && (
            <span> — <a href="/files">upload a journal file</a> first.</span>
          )}
          <button onClick={refetch} className="report-retry">Retry</button>
        </div>
      )}
      {!loading && !error && (
        <DataTable
          columns={COLUMNS}
          rows={entries}
          keyFn={(_, i) => i}
          emptyMessage="No transactions found for this filter."
        />
      )}
    </div>
  );
}
