import { useRef, useState } from 'react';
import { uploadFile } from '../api';
import type { FileInfo } from '../api';
import './FileUploader.css';

const ALLOWED = ['.journal', '.csv', '.rules'];

interface FileUploaderProps {
  onUploaded: (file: FileInfo) => void;
}

type State = 'idle' | 'uploading' | 'success' | 'error';

export function FileUploader({ onUploaded }: FileUploaderProps) {
  const [state, setState] = useState<State>('idle');
  const [error, setError] = useState('');
  const [dragging, setDragging] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  function validate(file: File): string | null {
    const ext = '.' + file.name.split('.').pop()?.toLowerCase();
    if (!ALLOWED.includes(ext)) {
      return `Unsupported file type "${ext}". Allowed: ${ALLOWED.join(', ')}`;
    }
    if (file.size > 10 * 1024 * 1024) {
      return 'File exceeds 10 MB limit.';
    }
    return null;
  }

  async function handleFile(file: File) {
    const err = validate(file);
    if (err) {
      setState('error');
      setError(err);
      return;
    }
    setState('uploading');
    setError('');
    try {
      const info = await uploadFile(file);
      setState('success');
      onUploaded(info);
      setTimeout(() => setState('idle'), 2000);
    } catch (e: unknown) {
      setState('error');
      setError(e instanceof Error ? e.message : 'Upload failed');
    }
  }

  function onDrop(e: React.DragEvent) {
    e.preventDefault();
    setDragging(false);
    const file = e.dataTransfer.files[0];
    if (file) handleFile(file);
  }

  function onInputChange(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (file) handleFile(file);
    e.target.value = '';
  }

  return (
    <div
      className={`file-uploader ${dragging ? 'file-uploader--dragging' : ''} file-uploader--${state}`}
      onDragOver={e => { e.preventDefault(); setDragging(true); }}
      onDragLeave={() => setDragging(false)}
      onDrop={onDrop}
      onClick={() => state === 'idle' && inputRef.current?.click()}
      role="button"
      tabIndex={0}
      onKeyDown={e => e.key === 'Enter' && inputRef.current?.click()}
    >
      <input
        ref={inputRef}
        type="file"
        accept={ALLOWED.join(',')}
        className="file-uploader-input"
        onChange={onInputChange}
      />

      {state === 'idle' && (
        <>
          <span className="file-uploader-icon">↑</span>
          <span className="file-uploader-label">Drop a file here or <u>click to browse</u></span>
          <span className="file-uploader-hint">.journal · .csv · .rules — max 10 MB</span>
        </>
      )}

      {state === 'uploading' && (
        <>
          <span className="file-uploader-spinner" />
          <span className="file-uploader-label">Uploading…</span>
        </>
      )}

      {state === 'success' && (
        <>
          <span className="file-uploader-icon file-uploader-icon--success">✓</span>
          <span className="file-uploader-label">Uploaded successfully</span>
        </>
      )}

      {state === 'error' && (
        <>
          <span className="file-uploader-icon file-uploader-icon--error">✗</span>
          <span className="file-uploader-label file-uploader-label--error">{error}</span>
          <button
            className="file-uploader-retry"
            onClick={e => { e.stopPropagation(); setState('idle'); setError(''); }}
            type="button"
          >
            Try again
          </button>
        </>
      )}
    </div>
  );
}
