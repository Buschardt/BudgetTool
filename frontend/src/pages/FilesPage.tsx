import { useState, useEffect } from 'react';
import { listFiles } from '../api';
import type { FileInfo } from '../api';
import { FileUploader } from '../components/FileUploader';
import { FileList } from '../components/FileList';
import { NewJournalModal } from '../components/NewJournalModal';
import './FilesPage.css';

export function FilesPage() {
  const [files, setFiles] = useState<FileInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [showNewJournal, setShowNewJournal] = useState(false);

  useEffect(() => {
    listFiles()
      .then(setFiles)
      .catch(e => setError(e instanceof Error ? e.message : 'Failed to load files'))
      .finally(() => setLoading(false));
  }, []);

  function handleUploaded(file: FileInfo) {
    setFiles(prev => [file, ...prev]);
  }

  function handleDeleted(id: number) {
    setFiles(prev => prev.filter(f => f.id !== id));
  }

  function handleConverted(file: FileInfo) {
    setFiles(prev => [file, ...prev]);
  }

  return (
    <div className="files-page">
      <h1>Files</h1>
      <p className="files-page-subtitle">
        Upload .journal, .csv, or .rules files. Convert CSV files to journal format for reporting.
      </p>

      <div className="files-page-actions">
        <FileUploader onUploaded={handleUploaded} />
        <div className="files-page-toolbar">
          <button
            type="button"
            className="files-page-new-journal-btn"
            onClick={() => setShowNewJournal(true)}
          >
            + New journal
          </button>
        </div>
      </div>

      {showNewJournal && (
        <NewJournalModal
          onCreated={file => { handleUploaded(file); setShowNewJournal(false); }}
          onCancel={() => setShowNewJournal(false)}
        />
      )}

      {loading && <p className="files-page-loading">Loading…</p>}
      {error && <p className="files-page-error">{error}</p>}
      {!loading && !error && (
        <FileList
          files={files}
          onDeleted={handleDeleted}
          onConverted={handleConverted}
        />
      )}
    </div>
  );
}
