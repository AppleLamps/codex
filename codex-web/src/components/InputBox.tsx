'use client';

import { useState, useRef, useEffect, KeyboardEvent } from 'react';
import { Send, Square, Paperclip } from 'lucide-react';
import clsx from 'clsx';

interface InputBoxProps {
  onSend: (message: string) => void;
  onInterrupt: () => void;
  disabled: boolean;
  isProcessing: boolean;
  placeholder?: string;
}

export function InputBox({
  onSend,
  onInterrupt,
  disabled,
  isProcessing,
  placeholder = 'Ask Codex to write code...',
}: InputBoxProps) {
  const [input, setInput] = useState('');
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Auto-resize textarea
  useEffect(() => {
    const textarea = textareaRef.current;
    if (textarea) {
      textarea.style.height = 'auto';
      textarea.style.height = Math.min(textarea.scrollHeight, 200) + 'px';
    }
  }, [input]);

  // Focus textarea on mount
  useEffect(() => {
    textareaRef.current?.focus();
  }, []);

  const handleSubmit = () => {
    const trimmed = input.trim();
    if (trimmed && !disabled && !isProcessing) {
      onSend(trimmed);
      setInput('');
    }
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  };

  return (
    <div className="border-t border-codex-border bg-codex-bg p-4">
      <div className="max-w-4xl mx-auto">
        <div className="relative bg-codex-surface border border-codex-border rounded-xl focus-within:border-codex-text transition-colors">
          <textarea
            ref={textareaRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={placeholder}
            disabled={disabled}
            rows={1}
            className="w-full bg-transparent px-4 py-3 pr-24 text-codex-text placeholder:text-codex-muted resize-none focus:outline-none disabled:opacity-50 font-mono text-sm"
          />
          <div className="absolute right-2 bottom-2 flex items-center gap-1">
            <button
              type="button"
              className="p-2 text-codex-muted hover:text-codex-text transition-colors disabled:opacity-50"
              disabled={disabled}
              title="Attach file"
            >
              <Paperclip size={18} />
            </button>
            {isProcessing ? (
              <button
                type="button"
                onClick={onInterrupt}
                className="p-2 bg-codex-error text-white rounded-lg hover:bg-red-600 transition-colors"
                title="Stop generation"
              >
                <Square size={18} />
              </button>
            ) : (
              <button
                type="button"
                onClick={handleSubmit}
                disabled={disabled || !input.trim()}
                className={clsx(
                  'p-2 rounded-lg transition-colors',
                  input.trim() && !disabled
                    ? 'bg-white text-black hover:bg-gray-100'
                    : 'bg-codex-border text-codex-muted cursor-not-allowed'
                )}
                title="Send message"
              >
                <Send size={18} />
              </button>
            )}
          </div>
        </div>
        <p className="mt-2 text-xs text-codex-muted text-center">
          Press Enter to send, Shift+Enter for new line
        </p>
      </div>
    </div>
  );
}
