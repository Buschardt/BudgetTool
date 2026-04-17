import { useState } from 'react';
import type { FileInfo } from '../api';
import { deleteFile, convertCsv } from '../api';
import { formatBytes } from '../lib/format';
import { ConvertModal } from './ConvertModal';
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
  const [modalCsvFile, setModalCsvFile] = useState<FileInfo | null>(null);

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

  function handleConvertClick(csvFile: FileInfo) {
    // Open the modal for the user to pick rules source
    setModalCsvFile(csvFile);
  }

  function handleModalCancel() {
    setModalCsvFile(null);
  }

  async function handleModalConfirm(rulesFileId?: number, rulesConfigId?: number) {
    if (!modalCsvFile) return;
    const csvId = modalCsvFile.id;
    setModalCsvFile(null);
    setConverting(csvId);
    setConvertError(prev => {
      const n = { ...prev };
      delete n[csvId];
      return n;
    });
    try {
      const result = await convertCsv(csvId, rulesFileId, rulesConfigId);
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
    <>
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
                  onClick={() => handleConvertClick(f)}
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

      {modalCsvFile && (
        <ConvertModal
          csvFile={modalCsvFile}
          rulesFiles={rulesFiles}
          onConfirm={handleModalConfirm}
          onCancel={handleModalCancel}
        />
      )}
    </>
  );
}
