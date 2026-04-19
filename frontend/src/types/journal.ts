export interface CommoditySetting {
  sample: string;
}

export interface AccountSetting {
  name: string;
}

export interface JournalSettings {
  default_commodity?: string | null;
  decimal_mark?: '.' | ',' | null;
  commodities: CommoditySetting[];
  accounts: AccountSetting[];
  includes: number[];
}

export interface JournalSettingsDetail {
  file: {
    id: number;
    filename: string;
    file_type: string;
    size_bytes: number;
    created_at: string;
  };
  settings: JournalSettings;
}

export const EMPTY_JOURNAL_SETTINGS: JournalSettings = {
  default_commodity: '',
  decimal_mark: null,
  commodities: [],
  accounts: [],
  includes: [],
};
