/**
 * Kriya Dispatcher — bridges Rust Host dispatch events to TypeScript action handlers.
 *
 * Flow:
 *   Rust emits "kriya:dispatch-action" event with { request_id, action_name, arguments }
 *   → Dispatcher executes the TS handler from the registry
 *   → Dispatcher sends the result back via kriya_dispatch_result IPC command
 */

import { registry } from './kriya-registry';

let unlistenDispatch: (() => void) | null = null;

export async function startDispatcher(): Promise<void> {
  if (typeof window === 'undefined' || !window.__TAURI__) return;

  const { listen } = window.__TAURI__.event;
  const { invoke } = window.__TAURI__.core;

  if (!listen || !invoke) return;

  // Prevent double-registration
  if (unlistenDispatch) {
    unlistenDispatch();
    unlistenDispatch = null;
  }

  unlistenDispatch = await listen('kriya:dispatch-action', async (event: any) => {
    const { request_id, action_name, arguments: args } = event.payload;

    let status = 'success';
    let output: any = null;
    let error_message: string | null = null;

    try {
      output = await registry.execute(action_name, args);
    } catch (err: any) {
      status = 'error';
      error_message = err?.message || String(err);
    }

    // Send result back to Rust Host
    try {
      await invoke('kriya_dispatch_result', {
        requestId: request_id,
        result: { status, output, error_message },
      });
    } catch (err) {
      console.error('[Kriya Dispatcher] Failed to send result back to Host:', err);
    }
  });

  console.log('[Kriya Dispatcher] Listening for dispatch events from Rust Host.');
}

export function stopDispatcher(): void {
  if (unlistenDispatch) {
    unlistenDispatch();
    unlistenDispatch = null;
    console.log('[Kriya Dispatcher] Stopped.');
  }
}
