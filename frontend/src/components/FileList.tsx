import { useState } from 'react';
import type { FileInfo } from '../api';
import { deleteFile, convertCsv } from '../api';
import { formatBytes } from '../lib/format';
import './FileList.css';

interface FileListProps {
  files: FileInfo[];
  onDeleted: (id: number) => void;
  onConverted: (file: FileInfo) => void;
}

export function FileList({ files, onDeleted, onConverted }: FileListProps) {
  const [converting, setConverting] = useState<number | null>(null);
  const [deleting, setDeleting] = useState<number | null>(null);
  const [convertError, setConvertError] = useState<Record<number, string>>({});

  const rulesFiles = files.filter(f => f.file_type === 'rules');

  async function handleDelete(id: number) {
    if (!confirm('Delete this file?')) return;
    setDeleting(id);
    try {
      await deleteFile(id);
      onDeleted(id);
    } catch (e: unknown) {
      alert(e instanceof Error ? e.message : 'Delete failed');
    } finally {
      setDeleting(null);
    }
  }

  async function handleConvert(csvFile: FileInfo) {
    // If there's exactly one rules file with a matching name, use it automatically
    const matchingRules = rulesFiles.find(r =>
      r.filename.replace(/\.rules$/, '') === csvFile.filename.replace(/\.csv$/, '')
    );
    const rulesId = matchingRules?.id ?? (rulesFiles.length === 1 ? rulesFiles[0].id : undefined);

    if (!rulesId && rulesFiles.length > 1) {
      // Ask user to pick
      const names = rulesFiles.map(r => `${r.id}: ${r.filename}`).join('\n');
      const input = prompt(`Enter the ID of the rules file to use:\n${names}`);
      if (!input) return;
      const picked = Number(input);
      if (!rulesFiles.find(r => r.id === picked)) { alert('Invalid ID'); return; }
      return doConvert(csvFile.id, picked);
    }

    return doConvert(csvFile.id, rulesId);
  }

  async function doConvert(csvId: number, rulesId?: number) {
    setConverting(csvId);
    setConvertError(prev => { const n = { ...prev }; delete n[csvId]; return n; });
    try {
      const result = await convertCsv(csvId, rulesId);
      onConverted(result);
    } catch (e: unknown) {
      setConvertError(prev => ({
        ...prev,
        [csvId]: e instanceof Error ? e.message : 'Conversion failed',
      }));
    } finally {
      setConverting(null);
    }
  }

  if (files.length === 0) {
    return <p className="file-list-empty">No files uploaded yet.</p>;
  }

  return (
    <ul className="file-list">
      {files.map(f => (
        <li key={f.id} className="file-list-item">
          <div className="file-list-info">
            <span className={`file-list-badge file-list-badge--${f.file_type}`}>
              {f.file_type}
            </span>
            <span className="file-list-name">{f.filename}</span>
            <span className="file-list-meta">
              {formatBytes(f.size_bytes)} · {f.created_at.slice(0, 10)}
            </span>
          </div>
          <div className="file-list-actions">
            {convertError[f.id] && (
              <span className="file-list-convert-error">{convertError[f.id]}</span>
            )}
            {f.file_type === 'csv' && (
              <button
                className="file-list-btn file-list-btn--convert"
                onClick={() => handleConvert(f)}
                disabled={converting === f.id}
                type="button"
              >
                {converting === f.id ? 'Converting…' : 'Convert'}
              </button>
            )}
            <button
              className="file-list-btn file-list-btn--delete"
              onClick={() => handleDelete(f.id)}
              disabled={deleting === f.id}
              type="button"
            >
              {deleting === f.id ? '…' : 'Delete'}
            </button>
          </div>
        </li>
      ))}
    </ul>
  );
}
