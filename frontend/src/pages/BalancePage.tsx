import { useState } from 'react';
import { getBalance } from '../api';
import type { ReportParams } from '../api';
import { useReport } from '../hooks/useReport';
import { parseBalance } from '../lib/hledger-parse';
import { formatCurrency } from '../lib/format';
import { ReportFilters } from '../components/ReportFilters';
import { DataTable } from '../components/DataTable';
import type { Column } from '../components/DataTable';
import type { AccountBalance } from '../types/hledger';
import {
  BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, Cell,
} from 'recharts';
import './BalancePage.css';

const COLUMNS: Column<AccountBalance>[] = [
  { key: 'account', header: 'Account' },
  {
    key: 'amount',
    header: 'Balance',
    align: 'right',
    render: r => (
      <span style={{ color: r.amount >= 0 ? 'var(--success)' : 'var(--danger)' }}>
        {formatCurrency(r.amount, r.commodity)}
      </span>
    ),
  },
];

export function BalancePage() {
  const [params, setParams] = useState<ReportParams>({});
  const { data: raw, loading, error, refetch } = useReport(getBalance, params);

  const balances = raw ? parseBalance(raw) : [];

  // For chart: top-level asset/liability accounts only
  const topLevel = balances.filter(b =>
    !b.account.includes(':') &&
    (b.account.startsWith('assets') || b.account.startsWith('liabilities'))
  );

  return (
    <div className="balance-page">
      <h1>Balance Sheet</h1>
      <ReportFilters onChange={setParams} />

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
        <div className="balance-page-content">
          {topLevel.length > 0 && (
            <div className="balance-page-chart">
              <h2>Account Summary</h2>
              <ResponsiveContainer width="100%" height={220}>
                <BarChart data={topLevel} margin={{ top: 8, right: 16, left: 0, bottom: 8 }}>
                  <XAxis
                    dataKey="account"
                    tick={{ fill: 'var(--text)', fontSize: 12 }}
                    axisLine={{ stroke: 'var(--border)' }}
                    tickLine={false}
                  />
                  <YAxis
                    tick={{ fill: 'var(--text)', fontSize: 12 }}
                    axisLine={false}
                    tickLine={false}
                    tickFormatter={v => formatCurrency(v as number)}
                  />
                  <Tooltip
                    contentStyle={{
                      background: 'var(--card-bg)',
                      border: '1px solid var(--border)',
                      borderRadius: '6px',
                      color: 'var(--text-h)',
                    }}
                    formatter={(value) => [formatCurrency(value as number), 'Balance']}
                  />
                  <Bar dataKey="amount" radius={[4, 4, 0, 0]}>
                    {topLevel.map((entry, i) => (
                      <Cell
                        key={i}
                        fill={entry.amount >= 0 ? '#34d399' : '#f87171'}
                      />
                    ))}
                  </Bar>
                </BarChart>
              </ResponsiveContainer>
            </div>
          )}

          <DataTable
            columns={COLUMNS}
            rows={balances}
            keyFn={(_, i) => i}
            emptyMessage="No balance data found."
          />
        </div>
      )}
    </div>
  );
}
