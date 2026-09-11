import { invoke, isTauri } from '@tauri-apps/api/core';

export interface AppInfo {
  name: string;
  version: string;
  environment: string;
  os: string;
  database_status: string;
  schema_version: number;
}

export interface AppStatus {
  is_tracking_paused: boolean;
  active_watchers_count: number;
  uptime_seconds: number;
}

export interface DatabaseStats {
  database_path: string;
  database_size_bytes: number;
  raw_events_count: number;
  blocks_count: number;
}

export interface IpcError {
  code: string;
  message: string;
}

export { isTauri };

export async function getAppInfo(): Promise<AppInfo> {
  if (!isTauri()) {
    return {
      name: 'tendly',
      version: '0.1.0-browser-preview',
      environment: 'browser-preview',
      os: 'unknown',
      database_status: 'uninitialized (browser mode)',
      schema_version: 0,
    };
  }
  return await invoke<AppInfo>('get_app_info');
}

export async function getAppStatus(): Promise<AppStatus> {
  if (!isTauri()) {
    return {
      is_tracking_paused: false,
      active_watchers_count: 0,
      uptime_seconds: 0,
    };
  }
  return await invoke<AppStatus>('get_app_status');
}

export async function getDatabaseStats(): Promise<DatabaseStats> {
  if (!isTauri()) {
    return {
      database_path: ':memory: (browser preview)',
      database_size_bytes: 0,
      raw_events_count: 0,
      blocks_count: 0,
    };
  }
  return await invoke<DatabaseStats>('get_database_stats');
}

export async function toggleTrackingPause(): Promise<boolean> {
  if (!isTauri()) {
    return false;
  }
  return await invoke<boolean>('toggle_tracking_pause');
}
