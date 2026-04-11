import { useMemo } from 'react';
import { getBalance, getIncomeStatement, getCashflow } from '../api';
import { useReport } from '../hooks/useReport';
import { parseBalance, parseIncomeStatement, parseCashflow } from '../lib/hledger-parse';
import { formatCurrency, toISODate, startOfMonth, monthsAgo } from '../lib/format';
import { SummaryCard } from '../components/SummaryCard';
import {
  AreaChart, Area, XAxis, YAxis, Tooltip, ResponsiveContainer, CartesianGrid,
  PieChart, Pie, Cell, Legend,
} from 'recharts';
import './DashboardPage.css';

const today = new Date();
const monthStart = toISODate(startOfMonth(today));
const todayStr = toISODate(today);
const yearAgo = toISODate(monthsAgo(12, today));

const MONTHLY_PARAMS = { begin: monthStart, end: todayStr };
const TREND_PARAMS = { begin: yearAgo, end: todayStr, period: 'monthly' };

const PIE_COLORS = ['#646cff', '#34d399', '#f87171', '#fbbf24', '#a78bfa', '#60a5fa', '#fb923c', '#4ade80'];

export function DashboardPage() {
  const { data: balRaw, loading: balLoading, error: balError } = useReport(getBalance);
  const { data: incRaw, loading: incLoading, error: incError } = useReport(getIncomeStatement, MONTHLY_PARAMS);
  const { data: trendRaw, loading: trendLoading } = useReport(getCashflow, TREND_PARAMS);

  const netWorth = useMemo(() => {
    if (!balRaw) return null;
    const rows = parseBalance(balRaw);
    return rows.reduce((sum, r) => sum + r.amount, 0);
  }, [balRaw]);

  const monthSummary = useMemo(() => {
    if (!incRaw) return null;
    return parseIncomeStatement(incRaw);
  }, [incRaw]);

  const trendData = useMemo(() => {
    if (!trendRaw) return [];
    return parseCashflow(trendRaw);
  }, [trendRaw]);

  const topExpenses = useMemo(() => {
    if (!monthSummary) return [];
    return monthSummary.expenses
      .slice()
      .sort((a, b) => Math.abs(b.amount) - Math.abs(a.amount))
      .slice(0, 6)
      .map(e => ({ name: e.account.split(':').pop() ?? e.account, value: Math.abs(e.amount) }));
  }, [monthSummary]);

  const loading = balLoading || incLoading || trendLoading;
  const hasErrors = balError && incError;

  if (loading) {
    return (
      <div className="dashboard-loading">
        <div className="dashboard-spinner" />
        <p>Loading dashboard…</p>
      </div>
    );
  }

  if (hasErrors) {
    return (
      <div className="dashboard-empty">
        <div className="dashboard-empty-icon">📂</div>
        <h2>No data yet</h2>
        <p>Upload a journal file to start seeing your finances.</p>
        <a href="/files" className="dashboard-empty-cta">Go to Files</a>
      </div>
    );
  }

  const monthlySavings = monthSummary
    ? monthSummary.revenues.reduce((s, r) => s + r.amount, 0)
      - monthSummary.expenses.reduce((s, r) => s + Math.abs(r.amount), 0)
    : null;

  return (
    <div className="dashboard">
      <div className="dashboard-cards">
        <SummaryCard
          title="Net Worth"
          value={netWorth !== null ? formatCurrency(netWorth) : '—'}
          trend={netWorth === null ? undefined : netWorth >= 0 ? 'up' : 'down'}
        />
        <SummaryCard
          title="Monthly Income"
          value={
            monthSummary
              ? formatCurrency(monthSummary.revenues.reduce((s, r) => s + r.amount, 0))
              : '—'
          }
          subtitle="this month"
          trend="up"
        />
        <SummaryCard
          title="Monthly Expenses"
          value={
            monthSummary
              ? formatCurrency(monthSummary.expenses.reduce((s, r) => s + Math.abs(r.amount), 0))
              : '—'
          }
          subtitle="this month"
          trend="down"
        />
        <SummaryCard
          title="Monthly Savings"
          value={monthlySavings !== null ? formatCurrency(monthlySavings) : '—'}
          subtitle="this month"
          trend={monthlySavings === null ? undefined : monthlySavings >= 0 ? 'up' : 'down'}
        />
      </div>

      <div className="dashboard-charts">
        <div className="dashboard-chart-card">
          <h2>Spending Trend (12 months)</h2>
          {trendData.length > 0 ? (
            <ResponsiveContainer width="100%" height={220}>
              <AreaChart data={trendData} margin={{ top: 8, right: 16, left: 0, bottom: 8 }}>
                <defs>
                  <linearGradient id="trendGrad" x1="0" y1="0" x2="0" y2="1">
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
                  formatter={(value) => [formatCurrency(value as number), 'Amount']}
                />
                <Area
                  type="monotone"
                  dataKey="amount"
                  stroke="#646cff"
                  strokeWidth={2}
                  fill="url(#trendGrad)"
                />
              </AreaChart>
            </ResponsiveContainer>
          ) : (
            <p className="dashboard-chart-empty">No trend data available.</p>
          )}
        </div>

        <div className="dashboard-chart-card">
          <h2>Expenses This Month</h2>
          {topExpenses.length > 0 ? (
            <ResponsiveContainer width="100%" height={220}>
              <PieChart>
                <Pie
                  data={topExpenses}
                  dataKey="value"
                  nameKey="name"
                  cx="50%"
                  cy="50%"
                  outerRadius={80}
                  strokeWidth={0}
                >
                  {topExpenses.map((_, i) => (
                    <Cell key={i} fill={PIE_COLORS[i % PIE_COLORS.length]} />
                  ))}
                </Pie>
                <Tooltip
                  contentStyle={{
                    background: 'var(--card-bg)',
                    border: '1px solid var(--border)',
                    borderRadius: '6px',
                    color: 'var(--text-h)',
                  }}
                  formatter={(value) => [formatCurrency(value as number), 'Expense']}
                />
                <Legend
                  formatter={value => (
                    <span style={{ color: 'var(--text)', fontSize: '0.8rem' }}>{value}</span>
                  )}
                />
              </PieChart>
            </ResponsiveContainer>
          ) : (
            <p className="dashboard-chart-empty">No expense data for this month.</p>
          )}
        </div>
      </div>
    </div>
  );
}
