import { useState, useEffect } from 'react';
import { listFiles, previewRulesConfig } from '../../api';
import type { FileInfo } from '../../api';
import './PreviewPanel.css';

interface Props {
  rulesConfigId: number | null; // null if not yet saved
  onSaveFirst: () => Promise<void>; // called to save before preview
}

export function PreviewPanel({ rulesConfigId, onSaveFirst }: Props) {
  const [csvFiles, setCsvFiles] = useState<FileInfo[]>([]);
  const [selectedCsvId, setSelectedCsvId] = useState<number | ''>('');
  const [previewing, setPreviewing] = useState(false);
  const [result, setResult] = useState('');
  const [error, setError] = useState('');
  const [open, setOpen] = useState(false);

  useEffect(() => {
    if (open) {
      listFiles()
        .then(files => setCsvFiles(files.filter(f => f.file_type === 'csv')))
        .catch(() => {});
    }
  }, [open]);

  async function handlePreview() {
    if (!selectedCsvId) return;
    setError('');
    setResult('');
    setPreviewing(true);

    try {
      // Save first if needed
      if (rulesConfigId === null) {
        await onSaveFirst();
        return; // onSaveFirst will update rulesConfigId; user needs to click preview again
      }
      const journal = await previewRulesConfig(rulesConfigId, Number(selectedCsvId));
      setResult(journal);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : 'Preview failed');
    } finally {
      setPreviewing(false);
    }
  }

  return (
    <section className="rule-section preview-panel">
      <button
        type="button"
        className="preview-toggle"
        onClick={() => setOpen(o => !o)}
      >
        <span className="preview-toggle-icon">{open ? '▾' : '▸'}</span>
        Preview
      </button>

      {open && (
        <div className="preview-body">
          <p className="preview-desc">
            Test your rules against an uploaded CSV file to see what journal entries will be
            generated. Save your config first, then pick a CSV and click Preview.
          </p>

          <div className="preview-controls">
            <select
              className="preview-csv-select"
              value={selectedCsvId}
              onChange={e => setSelectedCsvId(e.target.value === '' ? '' : Number(e.target.value))}
            >
              <option value="">Select a CSV file…</option>
              {csvFiles.map(f => (
                <option key={f.id} value={f.id}>
                  {f.filename}
                </option>
              ))}
            </select>

            <button
              type="button"
              className="preview-btn"
              onClick={handlePreview}
              disabled={!selectedCsvId || previewing}
            >
              {previewing ? 'Previewing…' : rulesConfigId === null ? 'Save first, then preview' : 'Preview'}
            </button>
          </div>

          {csvFiles.length === 0 && (
            <p className="preview-hint">No CSV files uploaded yet. Upload one on the Files page.</p>
          )}

          {error && <p className="preview-error">{error}</p>}

          {result && (
            <div className="preview-result">
              <div className="preview-result-header">
                <span>Generated journal entries</span>
                <button
                  type="button"
                  className="preview-clear-btn"
                  onClick={() => setResult('')}
                >
                  Clear
                </button>
              </div>
              <pre className="preview-output">{result}</pre>
            </div>
          )}
        </div>
      )}
    </section>
  );
}
