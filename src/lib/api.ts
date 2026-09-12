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

export interface WatcherStatus {
  name: string;
  supported: boolean;
  running: boolean;
  paused: boolean;
  last_error: string | null;
}

export interface CaptureStatus {
  is_tracking_paused: boolean;
  active_watchers_count: number;
  watchers: WatcherStatus[];
  total_raw_events: number;
  uptime_seconds: number;
}

export interface DatabaseStats {
  database_path: string;
  database_size_bytes: number;
  raw_events_count: number;
  blocks_count: number;
}

export type ActivityType = 'active' | 'afk' | 'unknown';

export interface TimeBlock {
  id: string;
  start_ms: number;
  end_ms: number;
  duration_ms: number;
  activity_type: ActivityType;
  dominant_app: string;
  dominant_title: string;
  dominant_url: string | null;
  classification: string | null;
  category: string | null;
  confidence: number | null;
  classified_by: string | null;
  user_override: string | null;
}

export interface CurrentActivityState {
  active_app: string;
  active_title: string;
  activity_type: ActivityType;
  current_block: TimeBlock | null;
  elapsed_in_state_seconds: number;
}

export interface AppDurationSummary {
  app: string;
  duration_ms: number;
}

export interface BrowserContext {
  browser: string;
  page_title: string | null;
  url: string | null;
  domain: string | null;
}

export interface ActivitySegment {
  start_ms: number;
  end_ms: number;
  app: string;
  title: string;
  activity_type: ActivityType;
  source: string;
  event_count: number;
  browser_context?: BrowserContext | null;
}

export interface ActivitySession {
  id: string;
  start_ms: number;
  end_ms: number;
  duration_ms: number;
  dominant_app: string;
  dominant_title: string;
  activity_type: ActivityType;
  block_count: number;
  time_blocks: TimeBlock[];
  has_secondary_activity: boolean;
  secondary_apps: AppDurationSummary[];
}

export interface SessionDetails {
  session: ActivitySession;
  segments: ActivitySegment[];
  app_breakdown: AppDurationSummary[];
}

export interface DailyTimeline {
  day_start_ms: number;
  day_end_ms: number;
  sessions: ActivitySession[];
  total_active_ms: number;
  total_afk_ms: number;
  total_unknown_ms: number;
  block_count: number;
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

export async function getCaptureStatus(): Promise<CaptureStatus> {
  if (!isTauri()) {
    return {
      is_tracking_paused: false,
      active_watchers_count: 0,
      watchers: [
        { name: 'watcher-x11', supported: false, running: false, paused: false, last_error: null },
        { name: 'watcher-wayland', supported: false, running: false, paused: false, last_error: null },
        { name: 'watcher-afk', supported: false, running: false, paused: false, last_error: null },
      ],
      total_raw_events: 0,
      uptime_seconds: 0,
    };
  }
  return await invoke<CaptureStatus>('get_capture_status');
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

export async function getCurrentActivity(): Promise<CurrentActivityState | null> {
  if (!isTauri()) {
    return {
      active_app: 'code',
      active_title: 'Tendly - VS Code (Browser Preview)',
      activity_type: 'active',
      current_block: null,
      elapsed_in_state_seconds: 42,
    };
  }
  return await invoke<CurrentActivityState | null>('get_current_activity');
}

export async function getRecentTimeBlocks(limit: number = 20): Promise<TimeBlock[]> {
  if (!isTauri()) {
    return [];
  }
  return await invoke<TimeBlock[]>('get_recent_time_blocks', { limit });
}

export async function reprocessTimeBlocks(): Promise<number> {
  if (!isTauri()) {
    return 0;
  }
  return await invoke<number>('reprocess_time_blocks');
}

export async function getDailyTimeline(
  dayStartMs: number,
  dayEndMs: number
): Promise<DailyTimeline> {
  if (!isTauri()) {
    const t0 = dayStartMs + 9 * 3600 * 1000; // 09:00
    const t1 = t0 + 45 * 60 * 1000; // 09:45
    const t2 = t1 + 20 * 60 * 1000; // 10:05
    const t3 = t2 + 35 * 60 * 1000; // 10:40

    return {
      day_start_ms: dayStartMs,
      day_end_ms: dayEndMs,
      sessions: [
        {
          id: `session:${t0}:${t1}`,
          start_ms: t0,
          end_ms: t1,
          duration_ms: 45 * 60 * 1000,
          dominant_app: 'code',
          dominant_title: 'src/lib/api.ts - tendly',
          activity_type: 'active',
          block_count: 15,
          time_blocks: [],
          has_secondary_activity: true,
          secondary_apps: [
            { app: 'firefox', duration_ms: 3 * 60 * 1000 },
            { app: 'slack', duration_ms: 2 * 60 * 1000 },
          ],
        },
        {
          id: `gap:${t1}:${t2}`,
          start_ms: t1,
          end_ms: t2,
          duration_ms: 20 * 60 * 1000,
          dominant_app: 'unrecorded',
          dominant_title: 'No recorded activity',
          activity_type: 'unknown',
          block_count: 0,
          time_blocks: [],
          has_secondary_activity: false,
          secondary_apps: [],
        },
        {
          id: `session:${t2}:${t3}`,
          start_ms: t2,
          end_ms: t3,
          duration_ms: 35 * 60 * 1000,
          dominant_app: 'firefox',
          dominant_title: 'GitHub - tendly/tendly: Pull Request #4',
          activity_type: 'active',
          block_count: 11,
          time_blocks: [],
          has_secondary_activity: false,
          secondary_apps: [],
        },
      ],
      total_active_ms: 80 * 60 * 1000,
      total_afk_ms: 0,
      total_unknown_ms: 20 * 60 * 1000,
      block_count: 26,
    };
  }
  return await invoke<DailyTimeline>('get_daily_timeline', {
    dayStartMs,
    dayEndMs,
  });
}

export async function getSessionDetails(
  sessionId: string,
  startMs: number,
  endMs: number
): Promise<SessionDetails> {
  if (!isTauri()) {
    return {
      session: {
        id: sessionId,
        start_ms: startMs,
        end_ms: endMs,
        duration_ms: endMs - startMs,
        dominant_app: 'code',
        dominant_title: 'src/lib/api.ts - tendly',
        activity_type: 'active',
        block_count: Math.ceil((endMs - startMs) / 180_000),
        time_blocks: [],
        has_secondary_activity: true,
        secondary_apps: [
          { app: 'firefox', duration_ms: 180_000 },
          { app: 'slack', duration_ms: 120_000 },
        ],
      },
      segments: [
        {
          start_ms: startMs,
          end_ms: startMs + 40 * 60 * 1000,
          app: 'code',
          title: 'src/lib/api.ts - tendly',
          activity_type: 'active',
          source: 'x11',
          event_count: 85,
        },
        {
          start_ms: startMs + 40 * 60 * 1000,
          end_ms: startMs + 43 * 60 * 1000,
          app: 'firefox',
          title: 'Svelte 5 Runes Documentation — Mozilla Firefox',
          activity_type: 'active',
          source: 'x11',
          event_count: 12,
          browser_context: {
            browser: 'firefox',
            page_title: 'Svelte 5 Runes Documentation',
            url: null,
            domain: null,
          },
        },
        {
          start_ms: startMs + 43 * 60 * 1000,
          end_ms: endMs,
          app: 'slack',
          title: '#general - Dev Chat',
          activity_type: 'active',
          source: 'x11',
          event_count: 8,
        },
      ],
      app_breakdown: [
        { app: 'code', duration_ms: 40 * 60 * 1000 },
        { app: 'firefox', duration_ms: 3 * 60 * 1000 },
        { app: 'slack', duration_ms: 2 * 60 * 1000 },
      ],
    };
  }
  return await invoke<SessionDetails>('get_session_details', {
    sessionId,
    startMs,
    endMs,
  });
}
