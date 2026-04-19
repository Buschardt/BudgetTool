import { useState, useEffect } from 'react';
import type {
  CommodityPriceSummary,
  ManualTransactionSummary,
  PeriodicTransactionSummary,
} from '../types/manual';
import { listPrices, listTransactions, listPeriodics } from '../api';
import { PriceForm } from '../components/manual/PriceForm';
import { PriceList } from '../components/manual/PriceList';
import { TransactionForm } from '../components/manual/TransactionForm';
import { TransactionList } from '../components/manual/TransactionList';
import { PeriodicForm } from '../components/manual/PeriodicForm';
import { PeriodicList } from '../components/manual/PeriodicList';
import './ManualEntryPage.css';

type Tab = 'prices' | 'transactions' | 'budgets';

export function ManualEntryPage() {
  const [tab, setTab] = useState<Tab>('prices');
  const [prices, setPrices] = useState<CommodityPriceSummary[]>([]);
  const [transactions, setTransactions] = useState<ManualTransactionSummary[]>([]);
  const [periodics, setPeriodics] = useState<PeriodicTransactionSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    let cancelled = false;
    Promise.all([listPrices(), listTransactions(), listPeriodics()])
      .then(([p, t, b]) => {
        if (cancelled) return;
        setPrices(p);
        setTransactions(t);
        setPeriodics(b);
        setLoading(false);
      })
      .catch(e => {
        if (cancelled) return;
        setError(e instanceof Error ? e.message : 'Failed to load entries');
        setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div className="manual-page">
      <div className="manual-page-header">
        <h1>Manual Entry</h1>
        <p className="manual-page-subtitle">
          Write P directives, budget rules, and one-off transactions directly into your journal.
        </p>
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

      {loading && <p className="manual-loading">Loading…</p>}
      {error && <p className="manual-error">{error}</p>}

      {!loading && !error && (
        <div className="manual-content">
          {tab === 'prices' && (
            <>
              <PriceForm
                editing={null}
                onSaved={p => setPrices(prev => [p, ...prev])}
                onCancel={() => {}}
              />
              <PriceList
                prices={prices}
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
                editing={null}
                onSaved={t => setTransactions(prev => [t, ...prev])}
                onCancel={() => {}}
              />
              <TransactionList
                transactions={transactions}
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
                editing={null}
                onSaved={p => setPeriodics(prev => [...prev, p])}
                onCancel={() => {}}
              />
              <PeriodicList
                periodics={periodics}
                onUpdated={updated =>
                  setPeriodics(prev => prev.map(p => (p.id === updated.id ? updated : p)))
                }
                onDeleted={id => setPeriodics(prev => prev.filter(p => p.id !== id))}
              />
            </>
          )}
        </div>
      )}
    </div>
  );
}
