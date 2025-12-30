'use client';

import { useEffect, useRef } from 'react';
import { MessageItem } from './MessageItem';
import { Loader2 } from 'lucide-react';
import type { ThreadItem, Turn } from '@/types/codex';

interface ChatThreadProps {
  items: ThreadItem[];
  turn: Turn | null;
  isLoading: boolean;
}

export function ChatThread({ items, turn, isLoading }: ChatThreadProps) {
  const bottomRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when new items arrive
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [items.length]);

  const isProcessing = turn?.status === 'inProgress';

  return (
    <div className="flex-1 overflow-y-auto">
      {items.length === 0 && !isLoading ? (
        <div className="h-full flex items-center justify-center">
          <div className="text-center max-w-md px-4">
            <div className="w-16 h-16 bg-white rounded-xl flex items-center justify-center mx-auto mb-4">
              <span className="text-black font-bold text-2xl">CX</span>
            </div>
            <h2 className="text-xl font-semibold text-codex-text mb-2">
              Welcome to Codex
            </h2>
            <p className="text-codex-muted">
              Start a conversation to get help with coding tasks, debugging, or exploring your codebase.
            </p>
          </div>
        </div>
      ) : (
        <div className="divide-y divide-codex-border">
          {items.map((item) => (
            <MessageItem key={item.id} item={item} />
          ))}
        </div>
      )}

      {/* Loading indicator */}
      {(isLoading || isProcessing) && (
        <div className="px-4 py-3 flex items-center gap-2 text-codex-muted">
          <Loader2 size={16} className="animate-spin" />
          <span className="text-sm">
            {isLoading ? 'Connecting...' : 'Processing...'}
          </span>
        </div>
      )}

      {/* Scroll anchor */}
      <div ref={bottomRef} />
    </div>
  );
}
