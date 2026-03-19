import { invoke, isTauri } from '@tauri-apps/api/core';

export default function request<R>(command: string, args?: Record<string, unknown>): Promise<R> {
  if (isTauri()) {
    return invoke<R>(command, args);
  }
  return Promise.reject(new Error('Not running in Tauri environment'));
}
