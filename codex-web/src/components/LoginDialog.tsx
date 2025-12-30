'use client';

import { useState } from 'react';
import { Key, Smartphone, X, Loader2, ExternalLink } from 'lucide-react';

interface LoginDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onLoginApiKey: (apiKey: string) => Promise<{ success: boolean; error?: string }>;
  onLoginDevice: () => Promise<{ userCode: string; verificationUri: string; expiresIn: number }>;
}

export function LoginDialog({
  isOpen,
  onClose,
  onLoginApiKey,
  onLoginDevice,
}: LoginDialogProps) {
  const [method, setMethod] = useState<'apiKey' | 'device' | null>(null);
  const [apiKey, setApiKey] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [deviceInfo, setDeviceInfo] = useState<{
    userCode: string;
    verificationUri: string;
    expiresIn: number;
  } | null>(null);

  if (!isOpen) return null;

  const handleApiKeySubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!apiKey.trim()) return;

    setLoading(true);
    setError(null);

    try {
      const result = await onLoginApiKey(apiKey);
      if (result.success) {
        onClose();
      } else {
        setError(result.error || 'Login failed');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Login failed');
    } finally {
      setLoading(false);
    }
  };

  const handleDeviceAuth = async () => {
    setLoading(true);
    setError(null);

    try {
      const info = await onLoginDevice();
      setDeviceInfo(info);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start device auth');
    } finally {
      setLoading(false);
    }
  };

  const resetDialog = () => {
    setMethod(null);
    setApiKey('');
    setError(null);
    setDeviceInfo(null);
    setLoading(false);
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-codex-surface border border-codex-border rounded-lg w-full max-w-md mx-4 overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-codex-border">
          <h2 className="text-lg font-semibold text-codex-text">
            {method === null ? 'Login to Codex' : method === 'apiKey' ? 'API Key Login' : 'Device Login'}
          </h2>
          <button
            onClick={() => {
              resetDialog();
              onClose();
            }}
            className="text-codex-muted hover:text-codex-text transition-colors"
          >
            <X size={20} />
          </button>
        </div>

        {/* Content */}
        <div className="p-4">
          {error && (
            <div className="mb-4 p-3 bg-codex-error/10 border border-codex-error/30 rounded-lg text-codex-error text-sm">
              {error}
            </div>
          )}

          {method === null && (
            <div className="space-y-3">
              <p className="text-codex-muted text-sm mb-4">
                Choose how you want to authenticate with Codex.
              </p>
              <button
                onClick={() => setMethod('apiKey')}
                className="w-full flex items-center gap-3 p-4 bg-codex-hover hover:bg-codex-border rounded-lg transition-colors text-left"
              >
                <Key className="text-codex-accent" size={24} />
                <div>
                  <p className="font-medium text-codex-text">API Key</p>
                  <p className="text-sm text-codex-muted">Use your OpenAI API key</p>
                </div>
              </button>
              <button
                onClick={() => {
                  setMethod('device');
                  handleDeviceAuth();
                }}
                className="w-full flex items-center gap-3 p-4 bg-codex-hover hover:bg-codex-border rounded-lg transition-colors text-left"
              >
                <Smartphone className="text-codex-accent" size={24} />
                <div>
                  <p className="font-medium text-codex-text">ChatGPT Account</p>
                  <p className="text-sm text-codex-muted">Login with your ChatGPT subscription</p>
                </div>
              </button>
            </div>
          )}

          {method === 'apiKey' && (
            <form onSubmit={handleApiKeySubmit} className="space-y-4">
              <div>
                <label htmlFor="apiKey" className="block text-sm font-medium text-codex-text mb-2">
                  OpenAI API Key
                </label>
                <input
                  id="apiKey"
                  type="password"
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  placeholder="sk-..."
                  className="w-full px-3 py-2 bg-codex-bg border border-codex-border rounded-lg text-codex-text placeholder-codex-muted focus:outline-none focus:ring-2 focus:ring-codex-accent"
                  autoFocus
                />
              </div>
              <div className="flex gap-2">
                <button
                  type="button"
                  onClick={resetDialog}
                  className="flex-1 px-4 py-2 border border-codex-border text-codex-text rounded-lg hover:bg-codex-hover transition-colors"
                >
                  Back
                </button>
                <button
                  type="submit"
                  disabled={loading || !apiKey.trim()}
                  className="flex-1 px-4 py-2 bg-codex-accent text-codex-bg rounded-lg font-medium hover:opacity-80 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                >
                  {loading && <Loader2 size={16} className="animate-spin" />}
                  Login
                </button>
              </div>
            </form>
          )}

          {method === 'device' && (
            <div className="space-y-4">
              {loading && !deviceInfo && (
                <div className="flex items-center justify-center py-8">
                  <Loader2 size={32} className="animate-spin text-codex-accent" />
                </div>
              )}

              {deviceInfo && (
                <>
                  <p className="text-codex-muted text-sm">
                    Open the link below and enter the code to complete login:
                  </p>
                  <div className="p-4 bg-codex-bg rounded-lg text-center">
                    <p className="text-2xl font-mono font-bold text-codex-text tracking-wider">
                      {deviceInfo.userCode}
                    </p>
                  </div>
                  <a
                    href={deviceInfo.verificationUri}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="flex items-center justify-center gap-2 px-4 py-2 bg-codex-accent text-codex-bg rounded-lg font-medium hover:opacity-80 transition-colors"
                  >
                    Open Login Page
                    <ExternalLink size={16} />
                  </a>
                  {deviceInfo.expiresIn && deviceInfo.expiresIn > 0 && (
                    <p className="text-xs text-codex-muted text-center">
                      Code expires in {Math.floor(deviceInfo.expiresIn / 60)} minutes
                    </p>
                  )}
                </>
              )}

              <button
                onClick={resetDialog}
                className="w-full px-4 py-2 border border-codex-border text-codex-text rounded-lg hover:bg-codex-hover transition-colors"
              >
                Back
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
