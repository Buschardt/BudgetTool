// TypeScript interfaces for manual journal entries.
// These mirror the Rust structs in api/src/models.rs (response shapes).

export interface Posting {
  account: string;
  amount?: string;
  commodity?: string;
  comment?: string;
}

// --- Prices ---

export interface CommodityPriceSummary {
  id: number;
  journal_file_id: number;
  date: string;
  commodity: string;
  amount: string;
  target_commodity: string;
  comment: string;
  created_at: string;
  updated_at: string;
}

export interface CreatePriceRequest {
  journal_file_id: number;
  date: string;
  commodity: string;
  amount: string;
  target_commodity: string;
  comment?: string;
}

export interface UpdatePriceRequest {
  date?: string;
  commodity?: string;
  amount?: string;
  target_commodity?: string;
  comment?: string;
}

export const EMPTY_PRICE: CreatePriceRequest = {
  journal_file_id: 0,
  date: '',
  commodity: '',
  amount: '',
  target_commodity: '',
  comment: '',
};

// --- Manual transactions ---

export interface ManualTransactionSummary {
  id: number;
  journal_file_id: number;
  date: string;
  status: string;
  code: string;
  description: string;
  comment: string;
  postings: Posting[];
  created_at: string;
  updated_at: string;
}

export interface CreateTransactionRequest {
  journal_file_id: number;
  date: string;
  status?: string;
  code?: string;
  description: string;
  comment?: string;
  postings: Posting[];
}

export interface UpdateTransactionRequest {
  date?: string;
  status?: string;
  code?: string;
  description?: string;
  comment?: string;
  postings?: Posting[];
}

export const EMPTY_TRANSACTION: CreateTransactionRequest = {
  journal_file_id: 0,
  date: '',
  status: '',
  code: '',
  description: '',
  comment: '',
  postings: [{ account: '', amount: '', commodity: '' }, { account: '' }],
};

// --- Periodic transactions (budgets) ---

export interface PeriodicTransactionSummary {
  id: number;
  journal_file_id: number;
  period: string;
  description: string;
  comment: string;
  postings: Posting[];
  created_at: string;
  updated_at: string;
}

export interface CreatePeriodicRequest {
  journal_file_id: number;
  period: string;
  description?: string;
  comment?: string;
  postings: Posting[];
}

export interface UpdatePeriodicRequest {
  period?: string;
  description?: string;
  comment?: string;
  postings?: Posting[];
}

export const EMPTY_PERIODIC: CreatePeriodicRequest = {
  journal_file_id: 0,
  period: '',
  description: '',
  comment: '',
  postings: [{ account: '', amount: '', commodity: '' }, { account: '' }],
};
