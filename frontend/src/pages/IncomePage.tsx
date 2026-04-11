import { useState } from 'react';
import { getIncomeStatement } from '../api';
import type { ReportParams } from '../api';
import { useReport } from '../hooks/useReport';
import { parseIncomeStatement } from '../lib/hledger-parse';
import { formatCurrency } from '../lib/format';
import { ReportFilters } from '../components/ReportFilters';
import { DataTable } from '../components/DataTable';
import type { Column } from '../components/DataTable';
import type { AccountBalance } from '../types/hledger';
import {
  BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, Cell,
} from 'recharts';
import './IncomePage.css';

const revenueColumns: Column<AccountBalance>[] = [
  { key: 'account', header: 'Account' },
  {
    key: 'amount',
    header: 'Amount',
    align: 'right',
    render: r => (
      <span style={{ color: 'var(--success)' }}>
        {formatCurrency(Math.abs(r.amount), r.commodity)}
      </span>
    ),
  },
];

const expenseColumns: Column<AccountBalance>[] = [
  { key: 'account', header: 'Account' },
  {
    key: 'amount',
    header: 'Amount',
    align: 'right',
    render: r => (
      <span style={{ color: 'var(--danger)' }}>
        {formatCurrency(Math.abs(r.amount), r.commodity)}
      </span>
    ),
  },
];

export function IncomePage() {
  const [params, setParams] = useState<ReportParams>({});
  const { data: raw, loading, error, refetch } = useReport(getIncomeStatement, params);

  const summary = raw ? parseIncomeStatement(raw) : null;

  const topExpenses = (summary?.expenses ?? [])
    .slice()
    .sort((a, b) => Math.abs(b.amount) - Math.abs(a.amount))
    .slice(0, 8);

  return (
    <div className="income-page">
      <h1>Income Statement</h1>
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

      {!loading && !error && summary && (
        <div className="income-page-content">
          {summary.netIncome !== 0 && (
            <div className="income-page-net">
              <span className="income-page-net-label">Net Income</span>
              <span
                className="income-page-net-value"
                style={{ color: summary.netIncome >= 0 ? 'var(--success)' : 'var(--danger)' }}
              >
                {formatCurrency(summary.netIncome)}
              </span>
            </div>
          )}

          <div className="income-page-tables">
            <div>
              <h2>Revenue</h2>
              <DataTable
                columns={revenueColumns}
                rows={summary.revenues}
                keyFn={(_, i) => i}
                emptyMessage="No revenue data."
              />
            </div>
            <div>
              <h2>Expenses</h2>
              <DataTable
                columns={expenseColumns}
                rows={summary.expenses}
                keyFn={(_, i) => i}
                emptyMessage="No expense data."
              />
            </div>
          </div>

          {topExpenses.length > 0 && (
            <div className="income-page-chart">
              <h2>Top Expense Categories</h2>
              <ResponsiveContainer width="100%" height={220}>
                <BarChart
                  data={topExpenses}
                  layout="vertical"
                  margin={{ top: 4, right: 24, left: 0, bottom: 4 }}
                >
                  <XAxis
                    type="number"
                    tick={{ fill: 'var(--text)', fontSize: 12 }}
                    axisLine={false}
                    tickLine={false}
                    tickFormatter={v => formatCurrency(v as number)}
                  />
                  <YAxis
                    type="category"
                    dataKey="account"
                    width={160}
                    tick={{ fill: 'var(--text)', fontSize: 12 }}
                    axisLine={false}
                    tickLine={false}
                  />
                  <Tooltip
                    contentStyle={{
                      background: 'var(--card-bg)',
                      border: '1px solid var(--border)',
                      borderRadius: '6px',
                      color: 'var(--text-h)',
                    }}
                    formatter={(value) => [formatCurrency(Math.abs(value as number)), 'Expense']}
                  />
                  <Bar dataKey="amount" radius={[0, 4, 4, 0]}>
                    {topExpenses.map((_, i) => (
                      <Cell key={i} fill="#f87171" />
                    ))}
                  </Bar>
                </BarChart>
              </ResponsiveContainer>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
