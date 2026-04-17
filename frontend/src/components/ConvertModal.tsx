import { useState, useEffect } from 'react';
import { listRulesConfigs } from '../api';
import type { FileInfo, RulesConfigSummary } from '../api';
import './ConvertModal.css';

interface Props {
  csvFile: FileInfo;
  rulesFiles: FileInfo[];
  onConfirm: (rulesFileId?: number, rulesConfigId?: number) => void;
  onCancel: () => void;
}

export function ConvertModal({ csvFile, rulesFiles, onConfirm, onCancel }: Props) {
  const [configs, setConfigs] = useState<RulesConfigSummary[]>([]);
  const [selected, setSelected] = useState<{ type: 'config' | 'file'; id: number } | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    listRulesConfigs()
      .then(c => {
        setConfigs(c);
        // Auto-select: prefer a config if only one exists and no rules files, or the only option
        const total = c.length + rulesFiles.length;
        if (total === 1) {
          if (c.length === 1) setSelected({ type: 'config', id: c[0].id });
          else setSelected({ type: 'file', id: rulesFiles[0].id });
        } else {
          // Try to auto-match rules file by stem
          const stem = csvFile.filename.replace(/\.csv$/i, '');
          const matchingFile = rulesFiles.find(
            r => r.filename.replace(/\.rules$/i, '') === stem
          );
          if (matchingFile) {
            setSelected({ type: 'file', id: matchingFile.id });
          }
        }
      })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [csvFile.filename, rulesFiles]);

  function handleConfirm() {
    if (!selected) return;
    if (selected.type === 'config') {
      onConfirm(undefined, selected.id);
    } else {
      onConfirm(selected.id, undefined);
    }
  }

  const hasOptions = configs.length > 0 || rulesFiles.length > 0;

  return (
    <div className="convert-modal-overlay" onClick={onCancel}>
      <div className="convert-modal" onClick={e => e.stopPropagation()}>
        <div className="convert-modal-header">
          <h2 className="convert-modal-title">Convert CSV</h2>
          <button
            type="button"
            className="convert-modal-close"
            onClick={onCancel}
          >
            ×
          </button>
        </div>

        <p className="convert-modal-file">
          Converting <strong>{csvFile.filename}</strong> using:
        </p>

        {loading && <p className="convert-modal-loading">Loading…</p>}

        {!loading && !hasOptions && (
          <div className="convert-modal-empty">
            <p>No rules configs or uploaded rules files found.</p>
            <p>
              Create a rules configuration on the <a href="/rules">Rules</a> page, or upload a{' '}
              <code>.rules</code> file on the Files page.
            </p>
          </div>
        )}

        {!loading && hasOptions && (
          <div className="convert-modal-options">
            {configs.length > 0 && (
              <div className="convert-modal-group">
                <div className="convert-modal-group-label">Rules Configurations</div>
                {configs.map(c => (
                  <label key={c.id} className="convert-modal-option">
                    <input
                      type="radio"
                      name="convert-source"
                      checked={selected?.type === 'config' && selected.id === c.id}
                      onChange={() => setSelected({ type: 'config', id: c.id })}
                    />
                    <div className="convert-modal-option-body">
                      <span className="convert-modal-option-name">{c.name}</span>
                      {c.description && (
                        <span className="convert-modal-option-desc">{c.description}</span>
                      )}
                    </div>
                    <span className="convert-modal-option-badge convert-modal-option-badge--config">
                      config
                    </span>
                  </label>
                ))}
              </div>
            )}

            {rulesFiles.length > 0 && (
              <div className="convert-modal-group">
                <div className="convert-modal-group-label">Uploaded Rules Files</div>
                {rulesFiles.map(f => (
                  <label key={f.id} className="convert-modal-option">
                    <input
                      type="radio"
                      name="convert-source"
                      checked={selected?.type === 'file' && selected.id === f.id}
                      onChange={() => setSelected({ type: 'file', id: f.id })}
                    />
                    <div className="convert-modal-option-body">
                      <span className="convert-modal-option-name">{f.filename}</span>
                    </div>
                    <span className="convert-modal-option-badge convert-modal-option-badge--file">
                      rules file
                    </span>
                  </label>
                ))}
              </div>
            )}
          </div>
        )}

        <div className="convert-modal-footer">
          <button
            type="button"
            className="convert-modal-confirm-btn"
            onClick={handleConfirm}
            disabled={!selected}
          >
            Convert
          </button>
          <button
            type="button"
            className="convert-modal-cancel-btn"
            onClick={onCancel}
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}
