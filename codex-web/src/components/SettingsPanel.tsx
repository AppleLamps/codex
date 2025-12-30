'use client';

import { useState, useEffect } from 'react';
import { X, Save, Loader2 } from 'lucide-react';

interface SettingsPanelProps {
  isOpen: boolean;
  onClose: () => void;
  sessionId: string | null;
}

interface ConfigValue {
  key: string;
  value: unknown;
  type: 'string' | 'boolean' | 'number' | 'select';
  label: string;
  description?: string;
  options?: { value: string; label: string }[];
}

const CONFIG_SCHEMA: ConfigValue[] = [
  {
    key: 'model',
    value: '',
    type: 'string',
    label: 'Default Model',
    description: 'The default model to use for conversations',
  },
  {
    key: 'approvalPolicy',
    value: 'onRequest',
    type: 'select',
    label: 'Approval Policy',
    description: 'When to require approval for tool executions',
    options: [
      { value: 'never', label: 'Never (Auto-approve all)' },
      { value: 'onRequest', label: 'On Request (Ask each time)' },
      { value: 'onFailure', label: 'On Failure (Ask only on errors)' },
      { value: 'unlessTrusted', label: 'Unless Trusted (Skip for trusted tools)' },
    ],
  },
  {
    key: 'sandboxMode',
    value: 'workspaceWrite',
    type: 'select',
    label: 'Sandbox Mode',
    description: 'File system access restrictions',
    options: [
      { value: 'readOnly', label: 'Read Only' },
      { value: 'workspaceWrite', label: 'Workspace Write' },
      { value: 'dangerFullAccess', label: 'Full Access (Dangerous)' },
    ],
  },
  {
    key: 'reasoningEffort',
    value: 'medium',
    type: 'select',
    label: 'Reasoning Effort',
    description: 'How much effort to put into reasoning',
    options: [
      { value: 'minimal', label: 'Minimal' },
      { value: 'low', label: 'Low' },
      { value: 'medium', label: 'Medium' },
      { value: 'high', label: 'High' },
      { value: 'xhigh', label: 'Extra High' },
    ],
  },
];

export function SettingsPanel({ isOpen, onClose, sessionId }: SettingsPanelProps) {
  const [config, setConfig] = useState<Record<string, unknown>>({});
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (isOpen && sessionId) {
      loadConfig();
    }
  }, [isOpen, sessionId]);

  const loadConfig = async () => {
    if (!sessionId) return;
    setLoading(true);
    setError(null);

    try {
      const response = await fetch(`/api/codex/config?sessionId=${sessionId}`);
      if (!response.ok) throw new Error('Failed to load config');
      const data = await response.json();
      setConfig(data || {});
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load config');
    } finally {
      setLoading(false);
    }
  };

  const saveConfig = async (key: string, value: unknown) => {
    if (!sessionId) return;
    setSaving(true);
    setError(null);

    try {
      const response = await fetch('/api/codex/config', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ sessionId, key, value }),
      });
      if (!response.ok) throw new Error('Failed to save config');
      setConfig((prev) => ({ ...prev, [key]: value }));
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save config');
    } finally {
      setSaving(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-codex-surface border border-codex-border rounded-lg w-full max-w-lg mx-4 max-h-[80vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-codex-border flex-shrink-0">
          <h2 className="text-lg font-semibold text-codex-text">Settings</h2>
          <button
            onClick={onClose}
            className="text-codex-muted hover:text-codex-text transition-colors"
          >
            <X size={20} />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-4">
          {error && (
            <div className="mb-4 p-3 bg-codex-error/10 border border-codex-error/30 rounded-lg text-codex-error text-sm">
              {error}
            </div>
          )}

          {loading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 size={32} className="animate-spin text-codex-accent" />
            </div>
          ) : (
            <div className="space-y-6">
              {CONFIG_SCHEMA.map((setting) => (
                <div key={setting.key}>
                  <label className="block text-sm font-medium text-codex-text mb-1">
                    {setting.label}
                  </label>
                  {setting.description && (
                    <p className="text-xs text-codex-muted mb-2">{setting.description}</p>
                  )}

                  {setting.type === 'string' && (
                    <input
                      type="text"
                      value={(config[setting.key] as string) || ''}
                      onChange={(e) => setConfig((prev) => ({ ...prev, [setting.key]: e.target.value }))}
                      onBlur={(e) => saveConfig(setting.key, e.target.value)}
                      className="w-full px-3 py-2 bg-codex-bg border border-codex-border rounded-lg text-codex-text placeholder-codex-muted focus:outline-none focus:ring-2 focus:ring-codex-accent"
                    />
                  )}

                  {setting.type === 'boolean' && (
                    <label className="flex items-center gap-2 cursor-pointer">
                      <input
                        type="checkbox"
                        checked={(config[setting.key] as boolean) || false}
                        onChange={(e) => {
                          setConfig((prev) => ({ ...prev, [setting.key]: e.target.checked }));
                          saveConfig(setting.key, e.target.checked);
                        }}
                        className="w-4 h-4 rounded border-codex-border bg-codex-bg text-codex-accent focus:ring-codex-accent"
                      />
                      <span className="text-sm text-codex-text">Enabled</span>
                    </label>
                  )}

                  {setting.type === 'select' && setting.options && (
                    <select
                      value={(config[setting.key] as string) || setting.options[0]?.value || ''}
                      onChange={(e) => {
                        setConfig((prev) => ({ ...prev, [setting.key]: e.target.value }));
                        saveConfig(setting.key, e.target.value);
                      }}
                      className="w-full px-3 py-2 bg-codex-bg border border-codex-border rounded-lg text-codex-text focus:outline-none focus:ring-2 focus:ring-codex-accent"
                    >
                      {setting.options.map((option) => (
                        <option key={option.value} value={option.value}>
                          {option.label}
                        </option>
                      ))}
                    </select>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="p-4 border-t border-codex-border flex-shrink-0">
          <div className="flex items-center justify-between">
            <p className="text-xs text-codex-muted">
              {saving ? 'Saving...' : 'Changes are saved automatically'}
            </p>
            <button
              onClick={onClose}
              className="px-4 py-2 bg-codex-accent text-codex-bg rounded-lg font-medium hover:opacity-80 transition-colors"
            >
              Done
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
