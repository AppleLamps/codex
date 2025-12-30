'use client';

import { useState, useRef, useEffect } from 'react';
import { ChevronDown, Check, Loader2 } from 'lucide-react';
import clsx from 'clsx';

interface Model {
  id: string;
  name: string;
  description?: string;
}

interface ModelSelectorProps {
  models: Model[];
  selectedModel: string | null;
  onSelectModel: (modelId: string) => void;
  loading?: boolean;
  disabled?: boolean;
}

export function ModelSelector({
  models,
  selectedModel,
  onSelectModel,
  loading = false,
  disabled = false,
}: ModelSelectorProps) {
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const selectedModelData = models.find((m) => m.id === selectedModel);

  return (
    <div className="relative" ref={dropdownRef}>
      <button
        onClick={() => !disabled && !loading && setIsOpen(!isOpen)}
        disabled={disabled || loading}
        className={clsx(
          'flex items-center gap-2 px-3 py-1.5 bg-codex-hover border border-codex-border rounded-lg text-sm transition-colors',
          disabled || loading
            ? 'opacity-50 cursor-not-allowed'
            : 'hover:bg-codex-border cursor-pointer'
        )}
      >
        {loading ? (
          <Loader2 size={14} className="animate-spin text-codex-muted" />
        ) : (
          <span className="text-codex-text truncate max-w-32">
            {selectedModelData?.name || 'Select model'}
          </span>
        )}
        <ChevronDown
          size={14}
          className={clsx('text-codex-muted transition-transform', isOpen && 'rotate-180')}
        />
      </button>

      {isOpen && (
        <div className="absolute top-full left-0 mt-1 w-64 bg-codex-surface border border-codex-border rounded-lg shadow-lg z-50 overflow-hidden">
          <div className="max-h-64 overflow-y-auto">
            {models.length === 0 ? (
              <p className="px-3 py-2 text-sm text-codex-muted">No models available</p>
            ) : (
              models.map((model) => (
                <button
                  key={model.id}
                  onClick={() => {
                    onSelectModel(model.id);
                    setIsOpen(false);
                  }}
                  className={clsx(
                    'w-full flex items-start gap-2 px-3 py-2 text-left hover:bg-codex-hover transition-colors',
                    selectedModel === model.id && 'bg-codex-hover'
                  )}
                >
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-codex-text truncate">{model.name}</p>
                    {model.description && (
                      <p className="text-xs text-codex-muted truncate">{model.description}</p>
                    )}
                  </div>
                  {selectedModel === model.id && (
                    <Check size={16} className="text-codex-accent flex-shrink-0 mt-0.5" />
                  )}
                </button>
              ))
            )}
          </div>
        </div>
      )}
    </div>
  );
}
