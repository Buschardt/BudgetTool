import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  getRulesConfig,
  createRulesConfig,
  updateRulesConfig,
  listRulesConfigs,
} from '../api';
import type { RulesConfig, RulesConfigSummary } from '../api';
import { EMPTY_RULES_CONFIG } from '../types/rules';
import { GeneralSettings } from '../components/rules/GeneralSettings';
import { FieldsMapping } from '../components/rules/FieldsMapping';
import { FieldAssignments } from '../components/rules/FieldAssignments';
import { ConditionalRulesEditor } from '../components/rules/ConditionalRulesEditor';
import { IncludesSelector } from '../components/rules/IncludesSelector';
import { PreviewPanel } from '../components/rules/PreviewPanel';
import '../components/rules/RuleSection.css';
import './RulesEditorPage.css';

export function RulesEditorPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const isNew = !id;

  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [config, setConfig] = useState<RulesConfig>({ ...EMPTY_RULES_CONFIG });
  const [allConfigs, setAllConfigs] = useState<RulesConfigSummary[]>([]);

  const [loading, setLoading] = useState(!isNew);
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState('');
  const [savedId, setSavedId] = useState<number | null>(null);

  useEffect(() => {
    // Fetch sibling configs for the Includes panel
    listRulesConfigs().then(setAllConfigs).catch(() => {});
  }, []);

  useEffect(() => {
    if (!id) return;
    setLoading(true);
    getRulesConfig(Number(id))
      .then(detail => {
        setName(detail.name);
        setDescription(detail.description);
        setConfig(detail.config);
        setSavedId(detail.id);
      })
      .catch(e => setSaveError(e instanceof Error ? e.message : 'Failed to load config'))
      .finally(() => setLoading(false));
  }, [id]);

  function patchConfig(patch: Partial<RulesConfig>) {
    setConfig(prev => ({ ...prev, ...patch }));
  }

  async function save(): Promise<number> {
    if (!name.trim()) {
      setSaveError('Name is required');
      throw new Error('Name is required');
    }
    setSaving(true);
    setSaveError('');
    try {
      if (isNew) {
        const detail = await createRulesConfig({ name, description, config });
        setSavedId(detail.id);
        navigate(`/rules/${detail.id}`, { replace: true });
        return detail.id;
      } else {
        const detail = await updateRulesConfig(Number(id), { name, description, config });
        setSavedId(detail.id);
        return detail.id;
      }
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : 'Save failed';
      setSaveError(msg);
      throw e;
    } finally {
      setSaving(false);
    }
  }

  async function handleSaveFirst() {
    await save();
  }

  const csvFields = config.fields ?? [];

  if (loading) {
    return (
      <div className="rules-editor-page">
        <p className="rules-editor-loading">Loading…</p>
      </div>
    );
  }

  return (
    <div className="rules-editor-page">
      {/* Header */}
      <div className="rules-editor-header">
        <div className="rules-editor-title-row">
          <button
            type="button"
            className="rules-editor-back-btn"
            onClick={() => navigate('/rules')}
          >
            ← Back
          </button>
          <h1 className="rules-editor-title">
            {isNew ? 'New Rules Config' : 'Edit Rules Config'}
          </h1>
        </div>

        <div className="rules-editor-meta">
          <div className="rules-editor-name-row">
            <input
              type="text"
              className="rules-editor-name-input"
              placeholder="Config name (e.g. Chase Checking)"
              value={name}
              onChange={e => setName(e.target.value)}
            />
            <textarea
              className="rules-editor-desc-input"
              placeholder="Description (optional)"
              rows={1}
              value={description}
              onChange={e => setDescription(e.target.value)}
            />
          </div>
          {saveError && <p className="rules-editor-save-error">{saveError}</p>}
          <div className="rules-editor-actions">
            <button
              type="button"
              className="rules-editor-save-btn"
              onClick={save}
              disabled={saving}
            >
              {saving ? 'Saving…' : isNew ? 'Create' : 'Save'}
            </button>
            <button
              type="button"
              className="rules-editor-cancel-btn"
              onClick={() => navigate('/rules')}
            >
              Cancel
            </button>
          </div>
        </div>
      </div>

      {/* Sections */}
      <div className="rules-editor-sections">
        <GeneralSettings config={config} onChange={patchConfig} />

        <FieldsMapping
          fields={csvFields}
          onChange={fields => patchConfig({ fields })}
        />

        <FieldAssignments
          assignments={config.assignments ?? []}
          onChange={assignments => patchConfig({ assignments })}
          csvFields={csvFields}
        />

        <ConditionalRulesEditor
          conditionals={config.conditionals ?? []}
          onChange={conditionals => patchConfig({ conditionals })}
          csvFields={csvFields}
        />

        <IncludesSelector
          includes={config.includes ?? []}
          allConfigs={allConfigs}
          currentId={savedId}
          onChange={includes => patchConfig({ includes })}
        />

        <PreviewPanel
          rulesConfigId={savedId}
          onSaveFirst={handleSaveFirst}
        />
      </div>
    </div>
  );
}
