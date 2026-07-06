/**
 * Kriya Initialization — registers default Noteriv actions and starts the dispatcher.
 * Call this once from page.tsx after the vault is loaded.
 */

import { registerDefaultNoterivActions } from './kriya-registry';
import { startDispatcher, stopDispatcher } from './kriya-dispatcher';

let initialized = false;

export async function initializeKriya(): Promise<void> {
  if (initialized) return;
  if (typeof window === 'undefined' || !window.electronAPI) return;

  // 1. Register default actions (sends metadata to Rust Host)
  registerDefaultNoterivActions();

  // 2. Start the dispatcher (listens for Rust → React events)
  await startDispatcher();

  initialized = true;
  console.log('[Kriya] Initialized — actions registered, dispatcher running.');
}

export function shutdownKriya(): void {
  stopDispatcher();
  initialized = false;
  console.log('[Kriya] Shutdown.');
}
