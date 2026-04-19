import { useState, useEffect } from 'react';
import type {
  CommodityPriceSummary,
  ManualTransactionSummary,
  PeriodicTransactionSummary,
} from '../types/manual';
import type { FileInfo } from '../api';
import { listFiles, getJournalSettings, listPrices, listTransactions, listPeriodics } from '../api';
import { PriceForm } from '../components/manual/PriceForm';
import { PriceList } from '../components/manual/PriceList';
import { TransactionForm } from '../components/manual/TransactionForm';
import { TransactionList } from '../components/manual/TransactionList';
import { PeriodicForm } from '../components/manual/PeriodicForm';
import { PeriodicList } from '../components/manual/PeriodicList';
import './ManualEntryPage.css';

const STORAGE_KEY = 'budgettool.selectedJournalId';

type Tab = 'prices' | 'transactions' | 'budgets';

export function ManualEntryPage() {
  const [tab, setTab] = useState<Tab>('prices');
  const [journals, setJournals] = useState<FileInfo[]>([]);
  const [selectedJournalId, setSelectedJournalId] = useState<number | null>(null);
  const [accounts, setAccounts] = useState<string[]>([]);
  const [prices, setPrices] = useState<CommodityPriceSummary[]>([]);
  const [transactions, setTransactions] = useState<ManualTransactionSummary[]>([]);
  const [periodics, setPeriodics] = useState<PeriodicTransactionSummary[]>([]);
  const [journalsLoading, setJournalsLoading] = useState(true);
  const [error, setError] = useState('');

  // Load the journal list once on mount
  useEffect(() => {
    let cancelled = false;
    listFiles()
      .then(files => {
        if (cancelled) return;
        const jnls = files.filter(f => f.file_type === 'journal');
        setJournals(jnls);
        if (jnls.length > 0) {
          const stored = localStorage.getItem(STORAGE_KEY);
          const storedId = stored ? Number(stored) : null;
          const found = storedId ? jnls.find(j => j.id === storedId) : null;
          setSelectedJournalId(found ? storedId : jnls[0].id);
        }
        setJournalsLoading(false);
      })
      .catch(e => {
        if (cancelled) return;
        setError(e instanceof Error ? e.message : 'Failed to load journals');
        setJournalsLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  // Reload settings and entries whenever the selected journal changes
  useEffect(() => {
    if (selectedJournalId === null) return;
    let cancelled = false;
    localStorage.setItem(STORAGE_KEY, String(selectedJournalId));
    Promise.all([
      getJournalSettings(selectedJournalId),
      listPrices(selectedJournalId),
      listTransactions(selectedJournalId),
      listPeriodics(selectedJournalId),
    ])
      .then(([detail, p, t, b]) => {
        if (cancelled) return;
        setAccounts(detail.settings.accounts.map(a => a.name));
        setPrices(p);
        setTransactions(t);
        setPeriodics(b);
        setError('');
      })
      .catch(e => {
        if (cancelled) return;
        setError(e instanceof Error ? e.message : 'Failed to load entries');
      });
    return () => {
      cancelled = true;
    };
  }, [selectedJournalId]);

  if (journalsLoading) {
    return (
      <div className="manual-page">
        <p className="manual-loading">Loading…</p>
      </div>
    );
  }

  if (journals.length === 0) {
    return (
      <div className="manual-page">
        <div className="manual-page-header">
          <h1>Manual Entry</h1>
          <p className="manual-page-subtitle">
            Write P directives, budget rules, and one-off transactions directly into your journal.
          </p>
        </div>
        <p className="manual-empty">
          No journals found.{' '}
          <a href="/files">Create a journal</a> first, then declare accounts in its settings.
        </p>
      </div>
    );
  }

  const journalId = selectedJournalId ?? journals[0].id;

  return (
    <div className="manual-page">
      <div className="manual-page-header">
        <h1>Manual Entry</h1>
        <p className="manual-page-subtitle">
          Write P directives, budget rules, and one-off transactions directly into your journal.
        </p>
      </div>

      <div className="manual-journal-bar">
        <label htmlFor="manual-journal-select" className="manual-journal-label">Journal</label>
        <select
          id="manual-journal-select"
          className="manual-journal-select"
          value={journalId}
          onChange={e => setSelectedJournalId(Number(e.target.value))}
        >
          {journals.map(j => (
            <option key={j.id} value={j.id}>{j.filename}</option>
          ))}
        </select>
      </div>

      <div className="manual-tabs">
        <button
          type="button"
          className={`manual-tab${tab === 'prices' ? ' manual-tab--active' : ''}`}
          onClick={() => setTab('prices')}
        >
          Prices
        </button>
        <button
          type="button"
          className={`manual-tab${tab === 'transactions' ? ' manual-tab--active' : ''}`}
          onClick={() => setTab('transactions')}
        >
          Transactions
        </button>
        <button
          type="button"
          className={`manual-tab${tab === 'budgets' ? ' manual-tab--active' : ''}`}
          onClick={() => setTab('budgets')}
        >
          Budgets
        </button>
      </div>

      {error && <p className="manual-error">{error}</p>}

      <div className="manual-content">
        {tab === 'prices' && (
          <>
            <PriceForm
              key={journalId}
              editing={null}
              journalId={journalId}
              onSaved={p => setPrices(prev => [p, ...prev])}
              onCancel={() => {}}
            />
            <PriceList
              key={`list-${journalId}`}
              prices={prices}
              journalId={journalId}
              onUpdated={updated =>
                setPrices(prev => prev.map(p => (p.id === updated.id ? updated : p)))
              }
              onDeleted={id => setPrices(prev => prev.filter(p => p.id !== id))}
            />
          </>
        )}

        {tab === 'transactions' && (
          <>
            <TransactionForm
              key={journalId}
              editing={null}
              journalId={journalId}
              accounts={accounts}
              onSaved={t => setTransactions(prev => [t, ...prev])}
              onCancel={() => {}}
            />
            <TransactionList
              key={`list-${journalId}`}
              transactions={transactions}
              journalId={journalId}
              accounts={accounts}
              onUpdated={updated =>
                setTransactions(prev => prev.map(t => (t.id === updated.id ? updated : t)))
              }
              onDeleted={id => setTransactions(prev => prev.filter(t => t.id !== id))}
            />
          </>
        )}

        {tab === 'budgets' && (
          <>
            <PeriodicForm
              key={journalId}
              editing={null}
              journalId={journalId}
              accounts={accounts}
              onSaved={p => setPeriodics(prev => [...prev, p])}
              onCancel={() => {}}
            />
            <PeriodicList
              key={`list-${journalId}`}
              periodics={periodics}
              journalId={journalId}
              accounts={accounts}
              onUpdated={updated =>
                setPeriodics(prev => prev.map(p => (p.id === updated.id ? updated : p)))
              }
              onDeleted={id => setPeriodics(prev => prev.filter(p => p.id !== id))}
            />
          </>
        )}
      </div>
    </div>
  );
}
