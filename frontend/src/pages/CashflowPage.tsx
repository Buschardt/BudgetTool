import { useState } from 'react';
import { getCashflow } from '../api';
import type { ReportParams } from '../api';
import { useReport } from '../hooks/useReport';
import { parseCashflow } from '../lib/hledger-parse';
import { formatCurrency } from '../lib/format';
import { ReportFilters } from '../components/ReportFilters';
import { DataTable } from '../components/DataTable';
import type { Column } from '../components/DataTable';
import type { AccountBalance } from '../types/hledger';
import {
  AreaChart, Area, XAxis, YAxis, Tooltip, ResponsiveContainer, CartesianGrid,
} from 'recharts';
import './CashflowPage.css';

const COLUMNS: Column<AccountBalance>[] = [
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
];

export function CashflowPage() {
  const [params, setParams] = useState<ReportParams>({});
  const { data: raw, loading, error, refetch } = useReport(getCashflow, params);

  const entries = raw ? parseCashflow(raw) : [];

  // When period is set, hledger returns periodic data — entries become chart-friendly
  const hasPeriod = !!params.period;

  return (
    <div className="cashflow-page">
      <h1>Cash Flow</h1>
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
        <div className="cashflow-page-content">
          {hasPeriod && entries.length > 0 && (
            <div className="cashflow-page-chart">
              <h2>Cash Flow Over Time</h2>
              <ResponsiveContainer width="100%" height={240}>
                <AreaChart data={entries} margin={{ top: 8, right: 16, left: 0, bottom: 8 }}>
                  <defs>
                    <linearGradient id="cfGrad" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#646cff" stopOpacity={0.3} />
                      <stop offset="95%" stopColor="#646cff" stopOpacity={0} />
                    </linearGradient>
                  </defs>
                  <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
                  <XAxis
                    dataKey="account"
                    tick={{ fill: 'var(--text)', fontSize: 11 }}
                    axisLine={{ stroke: 'var(--border)' }}
                    tickLine={false}
                  />
                  <YAxis
                    tick={{ fill: 'var(--text)', fontSize: 11 }}
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
                    formatter={(value) => [formatCurrency(value as number), 'Cash Flow']}
                  />
                  <Area
                    type="monotone"
                    dataKey="amount"
                    stroke="#646cff"
                    strokeWidth={2}
                    fill="url(#cfGrad)"
                  />
                </AreaChart>
              </ResponsiveContainer>
            </div>
          )}

          <DataTable
            columns={COLUMNS}
            rows={entries}
            keyFn={(_, i) => i}
            emptyMessage="No cash flow data found."
          />
        </div>
      )}
    </div>
  );
}
