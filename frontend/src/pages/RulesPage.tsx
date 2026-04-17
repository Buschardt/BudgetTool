import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { listRulesConfigs, deleteRulesConfig } from '../api';
import type { RulesConfigSummary } from '../api';
import './RulesPage.css';

export function RulesPage() {
  const [configs, setConfigs] = useState<RulesConfigSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [deleting, setDeleting] = useState<number | null>(null);

  useEffect(() => {
    listRulesConfigs()
      .then(setConfigs)
      .catch(e => setError(e instanceof Error ? e.message : 'Failed to load rules configs'))
      .finally(() => setLoading(false));
  }, []);

  async function handleDelete(id: number, name: string) {
    if (!confirm(`Delete rules config "${name}"?`)) return;
    setDeleting(id);
    try {
      await deleteRulesConfig(id);
      setConfigs(prev => prev.filter(c => c.id !== id));
    } catch (e: unknown) {
      alert(e instanceof Error ? e.message : 'Delete failed');
    } finally {
      setDeleting(null);
    }
  }

  return (
    <div className="rules-page">
      <div className="rules-page-header">
        <div>
          <h1>Rules Configurations</h1>
          <p className="rules-page-subtitle">
            Define how to import CSV files from your bank accounts into hledger journal format.
          </p>
        </div>
        <Link to="/rules/new" className="rules-page-new-btn">
          + New Config
        </Link>
      </div>

      {loading && <p className="rules-page-loading">Loading…</p>}
      {error && <p className="rules-page-error">{error}</p>}

      {!loading && !error && configs.length === 0 && (
        <div className="rules-page-empty">
          <p>No rules configurations yet.</p>
          <p>
            Create one to define how to map CSV columns from your bank to hledger accounts and
            fields.
          </p>
          <Link to="/rules/new" className="rules-page-new-btn">
            Create your first config
          </Link>
        </div>
      )}

      {!loading && configs.length > 0 && (
        <ul className="rules-config-list">
          {configs.map(c => (
            <li key={c.id} className="rules-config-card">
              <div className="rules-config-card-body">
                <Link to={`/rules/${c.id}`} className="rules-config-card-name">
                  {c.name}
                </Link>
                {c.description && (
                  <p className="rules-config-card-desc">{c.description}</p>
                )}
                <span className="rules-config-card-meta">
                  Updated {c.updated_at.slice(0, 10)}
                </span>
              </div>
              <div className="rules-config-card-actions">
                <Link to={`/rules/${c.id}`} className="rules-config-btn rules-config-btn--edit">
                  Edit
                </Link>
                <button
                  className="rules-config-btn rules-config-btn--delete"
                  onClick={() => handleDelete(c.id, c.name)}
                  disabled={deleting === c.id}
                  type="button"
                >
                  {deleting === c.id ? '…' : 'Delete'}
                </button>
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
