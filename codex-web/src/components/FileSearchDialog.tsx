'use client';

import { useState, useEffect, useCallback } from 'react';
import { Search, X, File, Loader2 } from 'lucide-react';

interface FileSearchDialogProps {
  isOpen: boolean;
  onClose: () => void;
  sessionId: string | null;
  onSelectFile?: (filePath: string) => void;
}

interface SearchResult {
  path: string;
  name: string;
  score?: number;
}

export function FileSearchDialog({
  isOpen,
  onClose,
  sessionId,
  onSelectFile,
}: FileSearchDialogProps) {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const search = useCallback(async (searchQuery: string) => {
    if (!sessionId || !searchQuery.trim()) {
      setResults([]);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const response = await fetch(
        `/api/codex/search?sessionId=${sessionId}&query=${encodeURIComponent(searchQuery)}&limit=20`
      );
      if (!response.ok) throw new Error('Search failed');
      const data = await response.json();
      setResults(data.results || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Search failed');
      setResults([]);
    } finally {
      setLoading(false);
    }
  }, [sessionId]);

  useEffect(() => {
    const timeoutId = setTimeout(() => {
      search(query);
    }, 300);

    return () => clearTimeout(timeoutId);
  }, [query, search]);

  useEffect(() => {
    if (!isOpen) {
      setQuery('');
      setResults([]);
      setError(null);
    }
  }, [isOpen]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-start justify-center pt-[10vh] z-50">
      <div className="bg-codex-surface border border-codex-border rounded-lg w-full max-w-xl mx-4 overflow-hidden">
        {/* Search Input */}
        <div className="flex items-center gap-2 p-3 border-b border-codex-border">
          <Search size={18} className="text-codex-muted" />
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search files..."
            className="flex-1 bg-transparent text-codex-text placeholder-codex-muted focus:outline-none"
            autoFocus
          />
          {loading && <Loader2 size={18} className="animate-spin text-codex-muted" />}
          <button
            onClick={onClose}
            className="text-codex-muted hover:text-codex-text transition-colors"
          >
            <X size={18} />
          </button>
        </div>

        {/* Results */}
        <div className="max-h-96 overflow-y-auto">
          {error && (
            <div className="p-3 text-sm text-codex-error">{error}</div>
          )}

          {!query.trim() && (
            <p className="p-4 text-sm text-codex-muted text-center">
              Start typing to search files
            </p>
          )}

          {query.trim() && results.length === 0 && !loading && !error && (
            <p className="p-4 text-sm text-codex-muted text-center">
              No files found
            </p>
          )}

          {results.map((result, index) => (
            <button
              key={index}
              onClick={() => {
                onSelectFile?.(result.path);
                onClose();
              }}
              className="w-full flex items-center gap-2 px-3 py-2 hover:bg-codex-hover transition-colors text-left"
            >
              <File size={16} className="text-codex-muted flex-shrink-0" />
              <div className="flex-1 min-w-0">
                <p className="text-sm text-codex-text truncate">{result.name}</p>
                <p className="text-xs text-codex-muted truncate">{result.path}</p>
              </div>
            </button>
          ))}
        </div>

        {/* Footer */}
        <div className="p-2 border-t border-codex-border text-xs text-codex-muted text-center">
          Press ESC to close
        </div>
      </div>
    </div>
  );
}
