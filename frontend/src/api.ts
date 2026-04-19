export interface FileInfo {
  id: number;
  filename: string;
  file_type: string;
  size_bytes: number;
  created_at: string;
}

export type { JournalSettings, JournalSettingsDetail } from './types/journal';
import type { JournalSettings, JournalSettingsDetail } from './types/journal';

export type { RulesConfigSummary, RulesConfigDetail, RulesConfig } from './types/rules';
import type { RulesConfigSummary, RulesConfigDetail, RulesConfig } from './types/rules';

export async function listRulesConfigs(): Promise<RulesConfigSummary[]> {
  return request<RulesConfigSummary[]>('/api/rules-configs');
}

export async function getRulesConfig(id: number): Promise<RulesConfigDetail> {
  return request<RulesConfigDetail>(`/api/rules-configs/${id}`);
}

export async function createRulesConfig(data: {
  name: string;
  description?: string;
  config: RulesConfig;
}): Promise<RulesConfigDetail> {
  return request<RulesConfigDetail>('/api/rules-configs', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
}

export async function updateRulesConfig(
  id: number,
  data: { name?: string; description?: string; config?: RulesConfig }
): Promise<RulesConfigDetail> {
  return request<RulesConfigDetail>(`/api/rules-configs/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
}

export async function deleteRulesConfig(id: number): Promise<void> {
  await request<string>(`/api/rules-configs/${id}`, { method: 'DELETE' });
}

export async function previewRulesConfig(
  id: number,
  csvFileId: number
): Promise<string> {
  return request<string>(`/api/rules-configs/${id}/preview`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ csv_file_id: csvFileId }),
  });
}

export type {
  CommodityPriceSummary,
  ManualTransactionSummary,
  PeriodicTransactionSummary,
  CreatePriceRequest,
  UpdatePriceRequest,
  CreateTransactionRequest,
  UpdateTransactionRequest,
  CreatePeriodicRequest,
  UpdatePeriodicRequest,
  Posting,
} from './types/manual';
import type {
  CommodityPriceSummary,
  ManualTransactionSummary,
  PeriodicTransactionSummary,
  CreatePriceRequest,
  UpdatePriceRequest,
  CreateTransactionRequest,
  UpdateTransactionRequest,
  CreatePeriodicRequest,
  UpdatePeriodicRequest,
} from './types/manual';

// --- Prices ---

export async function listPrices(): Promise<CommodityPriceSummary[]> {
  return request<CommodityPriceSummary[]>('/api/prices');
}

export async function createPrice(data: CreatePriceRequest): Promise<CommodityPriceSummary> {
  return request<CommodityPriceSummary>('/api/prices', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
}

export async function updatePrice(
  id: number,
  data: UpdatePriceRequest
): Promise<CommodityPriceSummary> {
  return request<CommodityPriceSummary>(`/api/prices/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
}

export async function deletePrice(id: number): Promise<void> {
  await request<string>(`/api/prices/${id}`, { method: 'DELETE' });
}

// --- Manual transactions ---

export async function listTransactions(): Promise<ManualTransactionSummary[]> {
  return request<ManualTransactionSummary[]>('/api/transactions');
}

export async function createTransaction(
  data: CreateTransactionRequest
): Promise<ManualTransactionSummary> {
  return request<ManualTransactionSummary>('/api/transactions', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
}

export async function updateTransaction(
  id: number,
  data: UpdateTransactionRequest
): Promise<ManualTransactionSummary> {
  return request<ManualTransactionSummary>(`/api/transactions/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
}

export async function deleteTransaction(id: number): Promise<void> {
  await request<string>(`/api/transactions/${id}`, { method: 'DELETE' });
}

// --- Periodic transactions ---

export async function listPeriodics(): Promise<PeriodicTransactionSummary[]> {
  return request<PeriodicTransactionSummary[]>('/api/periodics');
}

export async function createPeriodic(
  data: CreatePeriodicRequest
): Promise<PeriodicTransactionSummary> {
  return request<PeriodicTransactionSummary>('/api/periodics', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
}

export async function updatePeriodic(
  id: number,
  data: UpdatePeriodicRequest
): Promise<PeriodicTransactionSummary> {
  return request<PeriodicTransactionSummary>(`/api/periodics/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
}

export async function deletePeriodic(id: number): Promise<void> {
  await request<string>(`/api/periodics/${id}`, { method: 'DELETE' });
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

export async function createJournal(
  name: string,
  settings: JournalSettings
): Promise<FileInfo> {
  return request<FileInfo>('/api/journals', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name, settings }),
  });
}

export async function getJournalSettings(id: number): Promise<JournalSettingsDetail> {
  return request<JournalSettingsDetail>(`/api/journals/${id}/settings`);
}

export async function updateJournalSettings(
  id: number,
  settings: JournalSettings
): Promise<JournalSettingsDetail> {
  return request<JournalSettingsDetail>(`/api/journals/${id}/settings`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ settings }),
  });
}

export async function convertCsv(
  id: number,
  rulesFileId?: number,
  rulesConfigId?: number
): Promise<FileInfo> {
  return request<FileInfo>(`/api/files/${id}/convert`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      rules_file_id: rulesFileId ?? null,
      rules_config_id: rulesConfigId ?? null,
    }),
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
