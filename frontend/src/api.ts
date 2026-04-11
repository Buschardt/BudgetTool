export interface FileInfo {
  id: number;
  filename: string;
  file_type: string;
  size_bytes: number;
  created_at: string;
}

export interface ReportParams {
  begin?: string;
  end?: string;
  period?: string;
  depth?: number;
  account?: string;
}

function getToken(): string | null {
  return localStorage.getItem('token');
}

async function request<T>(
  path: string,
  options: RequestInit = {}
): Promise<T> {
  const token = getToken();
  const headers: Record<string, string> = {
    ...(options.headers as Record<string, string>),
  };
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const res = await fetch(path, { ...options, headers });
  const json = await res.json();

  if (!res.ok || !json.ok) {
    throw new Error(json.error ?? `Request failed: ${res.status}`);
  }

  return json.data as T;
}

export async function login(
  username: string,
  password: string
): Promise<{ token: string }> {
  return request<{ token: string }>('/api/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, password }),
  });
}

export async function uploadFile(file: File): Promise<FileInfo> {
  const body = new FormData();
  body.append('file', file);
  return request<FileInfo>('/api/files', { method: 'POST', body });
}

export async function listFiles(): Promise<FileInfo[]> {
  return request<FileInfo[]>('/api/files');
}

export async function getFile(id: number): Promise<FileInfo> {
  return request<FileInfo>(`/api/files/${id}`);
}

export async function deleteFile(id: number): Promise<void> {
  await request<string>(`/api/files/${id}`, { method: 'DELETE' });
}

export async function convertCsv(
  id: number,
  rulesFileId?: number
): Promise<FileInfo> {
  return request<FileInfo>(`/api/files/${id}/convert`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ rules_file_id: rulesFileId ?? null }),
  });
}

function buildQuery(params?: ReportParams): string {
  if (!params) return '';
  const q = new URLSearchParams();
  if (params.begin) q.set('begin', params.begin);
  if (params.end) q.set('end', params.end);
  if (params.period) q.set('period', params.period);
  if (params.depth !== undefined) q.set('depth', String(params.depth));
  if (params.account) q.set('account', params.account);
  const s = q.toString();
  return s ? `?${s}` : '';
}

export async function getBalance(params?: ReportParams): Promise<unknown> {
  return request<unknown>(`/api/reports/balance${buildQuery(params)}`);
}

export async function getIncomeStatement(params?: ReportParams): Promise<unknown> {
  return request<unknown>(`/api/reports/incomestatement${buildQuery(params)}`);
}

export async function getRegister(params?: ReportParams): Promise<unknown> {
  return request<unknown>(`/api/reports/register${buildQuery(params)}`);
}

export async function getCashflow(params?: ReportParams): Promise<unknown> {
  return request<unknown>(`/api/reports/cashflow${buildQuery(params)}`);
}
