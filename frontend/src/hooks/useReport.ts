import { useState, useEffect, useCallback, useRef } from 'react';
import type { ReportParams } from '../api';

interface ReportState<T> {
  data: T | null;
  loading: boolean;
  error: string | null;
}

export function useReport<T>(
  fetchFn: (params?: ReportParams) => Promise<T>,
  params?: ReportParams
): ReportState<T> & { refetch: () => void } {
  const [state, setState] = useState<ReportState<T>>({
    data: null,
    loading: true,
    error: null,
  });

  // Use a ref to hold the latest fetchFn so we don't re-run effect when it changes identity
  const fetchRef = useRef(fetchFn);
  fetchRef.current = fetchFn;

  const paramsKey = JSON.stringify(params ?? {});

  const run = useCallback(() => {
    setState(s => ({ ...s, loading: true, error: null }));
    fetchRef.current(params)
      .then(data => {
        console.log('[useReport] raw response:', data);
        setState({ data, loading: false, error: null });
      })
      .catch(err => setState({ data: null, loading: false, error: String(err?.message ?? err) }));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [paramsKey]);

  useEffect(() => {
    run();
  }, [run]);

  return { ...state, refetch: run };
}
