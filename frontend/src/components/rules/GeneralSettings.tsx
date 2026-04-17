import type { RulesConfig } from '../../types/rules';
import './GeneralSettings.css';

interface Props {
  config: RulesConfig;
  onChange: (patch: Partial<RulesConfig>) => void;
}

export function GeneralSettings({ config, onChange }: Props) {
  return (
    <section className="rule-section">
      <h2 className="rule-section-title">General Settings</h2>
      <p className="rule-section-desc">
        Configure how hledger reads the CSV file.
      </p>

      <div className="general-settings-grid">
        {/* Skip */}
        <div className="gs-field">
          <label className="gs-label" htmlFor="gs-skip">
            Skip header lines
          </label>
          <input
            id="gs-skip"
            type="number"
            min={0}
            max={100}
            className="gs-input"
            placeholder="1"
            value={config.skip ?? ''}
            onChange={e =>
              onChange({ skip: e.target.value === '' ? undefined : Number(e.target.value) })
            }
          />
          <span className="gs-hint">Number of non-empty lines to skip at the top of the CSV</span>
        </div>

        {/* Separator */}
        <div className="gs-field">
          <label className="gs-label" htmlFor="gs-separator">
            Field separator
          </label>
          <select
            id="gs-separator"
            className="gs-select"
            value={config.separator ?? ''}
            onChange={e =>
              onChange({
                separator: (e.target.value as RulesConfig['separator']) || undefined,
              })
            }
          >
            <option value="">Auto-detect from file extension</option>
            <option value="comma">Comma (,)</option>
            <option value="semicolon">Semicolon (;)</option>
            <option value="tab">Tab</option>
            <option value="space">Space</option>
          </select>
        </div>

        {/* Date format */}
        <div className="gs-field">
          <label className="gs-label" htmlFor="gs-datefmt">
            Date format
          </label>
          <input
            id="gs-datefmt"
            type="text"
            className="gs-input"
            placeholder="%Y-%m-%d"
            value={config.dateFormat ?? ''}
            onChange={e => onChange({ dateFormat: e.target.value || undefined })}
          />
          <span className="gs-hint">
            strptime pattern, e.g. <code>%d/%m/%Y</code>, <code>%m/%d/%y</code>. Leave blank for
            ISO dates (YYYY-MM-DD).
          </span>
        </div>

        {/* Decimal mark */}
        <div className="gs-field">
          <label className="gs-label" htmlFor="gs-decimal">
            Decimal mark
          </label>
          <select
            id="gs-decimal"
            className="gs-select"
            value={config.decimalMark ?? ''}
            onChange={e =>
              onChange({
                decimalMark: (e.target.value as RulesConfig['decimalMark']) || undefined,
              })
            }
          >
            <option value="">Auto-detect</option>
            <option value=".">Period (.)</option>
            <option value=",">Comma (,)</option>
          </select>
          <span className="gs-hint">
            Required when amounts use digit group separators (e.g. 1,234.56)
          </span>
        </div>

        {/* Balance type */}
        <div className="gs-field">
          <label className="gs-label" htmlFor="gs-balancetype">
            Balance assertion type
          </label>
          <select
            id="gs-balancetype"
            className="gs-select"
            value={config.balanceType ?? ''}
            onChange={e =>
              onChange({
                balanceType: (e.target.value as RulesConfig['balanceType']) || undefined,
              })
            }
          >
            <option value="">Default (=)</option>
            <option value="=">= Single commodity, exclude subaccounts</option>
            <option value="=*">=* Single commodity, include subaccounts</option>
            <option value="==">== Multi commodity, exclude subaccounts</option>
            <option value="==*">==* Multi commodity, include subaccounts</option>
          </select>
        </div>

        {/* Encoding */}
        <div className="gs-field">
          <label className="gs-label" htmlFor="gs-encoding">
            Encoding
          </label>
          <input
            id="gs-encoding"
            type="text"
            className="gs-input"
            placeholder="utf-8"
            value={config.encoding ?? ''}
            onChange={e => onChange({ encoding: e.target.value || undefined })}
          />
          <span className="gs-hint">
            Text encoding of the CSV file. Common values: utf-8, iso-8859-1, cp1252
          </span>
        </div>

        {/* Timezone */}
        <div className="gs-field">
          <label className="gs-label" htmlFor="gs-timezone">
            Timezone
          </label>
          <input
            id="gs-timezone"
            type="text"
            className="gs-input"
            placeholder="UTC"
            value={config.timezone ?? ''}
            onChange={e => onChange({ timezone: e.target.value || undefined })}
          />
          <span className="gs-hint">
            Timezone for date-times lacking one. Examples: UTC, EST, +0530
          </span>
        </div>

        {/* Checkboxes */}
        <div className="gs-field gs-field--checkboxes">
          <label className="gs-checkbox-label">
            <input
              type="checkbox"
              checked={config.newestFirst ?? false}
              onChange={e => onChange({ newestFirst: e.target.checked || undefined })}
            />
            Newest transactions first (within same day)
          </label>
          <label className="gs-checkbox-label">
            <input
              type="checkbox"
              checked={config.intraDayReversed ?? false}
              onChange={e => onChange({ intraDayReversed: e.target.checked || undefined })}
            />
            Intra-day order reversed from file order
          </label>
        </div>
      </div>
    </section>
  );
}
