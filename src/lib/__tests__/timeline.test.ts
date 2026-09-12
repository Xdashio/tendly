import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import Timeline from '../components/Timeline.svelte';
import * as api from '../api';

vi.mock('../api', async (importOriginal) => {
  const actual = await importOriginal<typeof api>();
  return {
    ...actual,
    getDailyTimeline: vi.fn(),
    getSessionDetails: vi.fn(),
  };
});

describe('Timeline Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders timeline with sessions, dominant app, and duration', async () => {
    vi.mocked(api.getDailyTimeline).mockResolvedValueOnce({
      day_start_ms: 1700000000000,
      day_end_ms: 1700086400000,
      sessions: [
        {
          id: 'session:1',
          start_ms: 1700032800000,
          end_ms: 1700036400000,
          duration_ms: 3600000,
          dominant_app: 'code',
          dominant_title: 'src/main.rs - tendly',
          activity_type: 'active',
          block_count: 12,
          time_blocks: [],
          has_secondary_activity: false,
          secondary_apps: [],
        },
      ],
      total_active_ms: 3600000,
      total_afk_ms: 0,
      total_unknown_ms: 0,
      block_count: 12,
    });

    const { findByText, findAllByText } = render(Timeline, { currentActivity: null });

    expect(await findByText('code')).toBeTruthy();
    expect(await findByText('src/main.rs - tendly')).toBeTruthy();
    expect((await findAllByText('1h')).length).toBeGreaterThanOrEqual(1);
    expect(await findByText('ACTIVE')).toBeTruthy();
  });

  it('renders empty state when there is no activity for the date', async () => {
    vi.mocked(api.getDailyTimeline).mockResolvedValueOnce({
      day_start_ms: 1700000000000,
      day_end_ms: 1700086400000,
      sessions: [],
      total_active_ms: 0,
      total_afk_ms: 0,
      total_unknown_ms: 0,
      block_count: 0,
    });

    const { findByText } = render(Timeline, { currentActivity: null });

    expect(await findByText(/No recorded activity for/)).toBeTruthy();
  });

  it('renders secondary activity preview when present', async () => {
    vi.mocked(api.getDailyTimeline).mockResolvedValueOnce({
      day_start_ms: 1700000000000,
      day_end_ms: 1700086400000,
      sessions: [
        {
          id: 'session:2',
          start_ms: 1700032800000,
          end_ms: 1700036400000,
          duration_ms: 3600000,
          dominant_app: 'code',
          dominant_title: 'src/main.rs',
          activity_type: 'active',
          block_count: 12,
          time_blocks: [],
          has_secondary_activity: true,
          secondary_apps: [
            { app: 'firefox', duration_ms: 180000 },
            { app: 'slack', duration_ms: 120000 },
          ],
        },
      ],
      total_active_ms: 3600000,
      total_afk_ms: 0,
      total_unknown_ms: 0,
      block_count: 12,
    });

    const { findByText } = render(Timeline, { currentActivity: null });

    expect(await findByText('Includes:')).toBeTruthy();
    expect(await findByText(/firefox \(3m\)/)).toBeTruthy();
    expect(await findByText(/slack \(2m\)/)).toBeTruthy();
  });

  it('expands session and retrieves drill-down composition on demand', async () => {
    const session = {
      id: 'session:drill',
      start_ms: 1700032800000,
      end_ms: 1700036400000,
      duration_ms: 3600000,
      dominant_app: 'code',
      dominant_title: 'src/main.rs',
      activity_type: 'active' as const,
      block_count: 12,
      time_blocks: [],
      has_secondary_activity: true,
      secondary_apps: [{ app: 'firefox', duration_ms: 600000 }],
    };

    vi.mocked(api.getDailyTimeline).mockResolvedValueOnce({
      day_start_ms: 1700000000000,
      day_end_ms: 1700086400000,
      sessions: [session],
      total_active_ms: 3600000,
      total_afk_ms: 0,
      total_unknown_ms: 0,
      block_count: 12,
    });

    vi.mocked(api.getSessionDetails).mockResolvedValueOnce({
      session,
      segments: [
        {
          start_ms: 1700032800000,
          end_ms: 1700035800000,
          app: 'code',
          title: 'src/main.rs',
          activity_type: 'active',
          source: 'x11',
          event_count: 50,
        },
        {
          start_ms: 1700035800000,
          end_ms: 1700036400000,
          app: 'firefox',
          title: 'Docs',
          activity_type: 'active',
          source: 'x11',
          event_count: 10,
        },
      ],
      app_breakdown: [
        { app: 'code', duration_ms: 3000000 },
        { app: 'firefox', duration_ms: 600000 },
      ],
    });

    const { findByText } = render(Timeline, { currentActivity: null });
    const inspectBtn = await findByText('Inspect [+]');
    await fireEvent.click(inspectBtn);

    expect(api.getSessionDetails).toHaveBeenCalledWith('session:drill', 1700032800000, 1700036400000);
    expect(await findByText('Session Application Breakdown')).toBeTruthy();
    expect(await findByText('Hide Details [-]')).toBeTruthy();
  });

  it('integrates current activity indicator when on today', async () => {
    vi.mocked(api.getDailyTimeline).mockResolvedValueOnce({
      day_start_ms: 1700000000000,
      day_end_ms: 1700086400000,
      sessions: [],
      total_active_ms: 0,
      total_afk_ms: 0,
      total_unknown_ms: 0,
      block_count: 0,
    });

    const currentActivity = {
      active_app: 'terminal',
      active_title: 'cargo build',
      activity_type: 'active' as const,
      current_block: null,
      elapsed_in_state_seconds: 15,
    };

    const { findByText } = render(Timeline, { currentActivity });

    expect(await findByText('Current activity:')).toBeTruthy();
    expect(await findByText('terminal')).toBeTruthy();
    expect(await findByText('(cargo build)')).toBeTruthy();
    expect(await findByText('15s in state')).toBeTruthy();
  });

  it('renders error state on API failure and retries on request', async () => {
    vi.mocked(api.getDailyTimeline).mockRejectedValueOnce(new Error('IPC query failed'));

    const { findByText } = render(Timeline, { currentActivity: null });

    expect(await findByText('Timeline Error')).toBeTruthy();
    expect(await findByText('IPC query failed')).toBeTruthy();

    vi.mocked(api.getDailyTimeline).mockResolvedValueOnce({
      day_start_ms: 1700000000000,
      day_end_ms: 1700086400000,
      sessions: [],
      total_active_ms: 0,
      total_afk_ms: 0,
      total_unknown_ms: 0,
      block_count: 0,
    });

    const retryBtn = await findByText('Retry');
    await fireEvent.click(retryBtn);

    await waitFor(() => {
      expect(api.getDailyTimeline).toHaveBeenCalledTimes(2);
    });
  });

  it('navigates dates with Prev button', async () => {
    vi.mocked(api.getDailyTimeline).mockResolvedValue({
      day_start_ms: 1700000000000,
      day_end_ms: 1700086400000,
      sessions: [],
      total_active_ms: 0,
      total_afk_ms: 0,
      total_unknown_ms: 0,
      block_count: 0,
    });

    const { findByTitle } = render(Timeline, { currentActivity: null });
    const prevBtn = await findByTitle('Previous Day');
    await fireEvent.click(prevBtn);

    await waitFor(() => {
      expect(api.getDailyTimeline).toHaveBeenCalledTimes(2);
    });
  });

  it('does not create duplicate session cards when current activity is active', async () => {
    vi.mocked(api.getDailyTimeline).mockResolvedValueOnce({
      day_start_ms: 1700000000000,
      day_end_ms: 1700086400000,
      sessions: [
        {
          id: 'session:current_overlap',
          start_ms: 1700032800000,
          end_ms: 1700036400000,
          duration_ms: 3600000,
          dominant_app: 'code',
          dominant_title: 'src/main.rs',
          activity_type: 'active',
          block_count: 12,
          time_blocks: [],
          has_secondary_activity: false,
          secondary_apps: [],
        },
      ],
      total_active_ms: 3600000,
      total_afk_ms: 0,
      total_unknown_ms: 0,
      block_count: 12,
    });

    const currentActivity = {
      active_app: 'code',
      active_title: 'src/main.rs',
      activity_type: 'active' as const,
      current_block: null,
      elapsed_in_state_seconds: 45,
    };

    const { findAllByText, findByText } = render(Timeline, { currentActivity });

    // Live banner has 'Current activity:'
    expect(await findByText('Current activity:')).toBeTruthy();
    // Verify that exactly 1 session card exists with dominant title 'src/main.rs' (no duplicate session card)
    const matches = await findAllByText('src/main.rs');
    expect(matches.length).toBe(1);
  });

  it('displays idle notification when current activity is stale or unknown', async () => {
    vi.mocked(api.getDailyTimeline).mockResolvedValueOnce({
      day_start_ms: 1700000000000,
      day_end_ms: 1700086400000,
      sessions: [],
      total_active_ms: 0,
      total_afk_ms: 0,
      total_unknown_ms: 0,
      block_count: 0,
    });

    const currentActivity = {
      active_app: 'stale_app',
      active_title: 'stale_title',
      activity_type: 'unknown' as const,
      current_block: null,
      elapsed_in_state_seconds: 300,
    };

    const { findByText } = render(Timeline, { currentActivity });

    expect(await findByText('No active window input detected')).toBeTruthy();
    expect(await findByText('300s in state')).toBeTruthy();
  });

  it('disables Next button when viewing today to guard against future navigation', async () => {
    vi.mocked(api.getDailyTimeline).mockResolvedValue({
      day_start_ms: 1700000000000,
      day_end_ms: 1700086400000,
      sessions: [],
      total_active_ms: 0,
      total_afk_ms: 0,
      total_unknown_ms: 0,
      block_count: 0,
    });

    const { findByTitle } = render(Timeline, { currentActivity: null });
    const nextBtn = await findByTitle('Next Day');

    expect(nextBtn.hasAttribute('disabled')).toBe(true);
  });

  it('renders browser context page_title and domain in drill-down segments when present', async () => {
    const session = {
      id: 'session:browser_drill',
      start_ms: 1700032800000,
      end_ms: 1700036400000,
      duration_ms: 3600000,
      dominant_app: 'firefox',
      dominant_title: 'GitHub - tendly/tendly: Pull Request #5 — Mozilla Firefox',
      activity_type: 'active' as const,
      block_count: 12,
      time_blocks: [],
      has_secondary_activity: false,
      secondary_apps: [],
    };

    vi.mocked(api.getDailyTimeline).mockResolvedValueOnce({
      day_start_ms: 1700000000000,
      day_end_ms: 1700086400000,
      sessions: [session],
      total_active_ms: 3600000,
      total_afk_ms: 0,
      total_unknown_ms: 0,
      block_count: 12,
    });

    vi.mocked(api.getSessionDetails).mockResolvedValueOnce({
      session,
      segments: [
        {
          start_ms: 1700032800000,
          end_ms: 1700036400000,
          app: 'firefox',
          title: 'GitHub - tendly/tendly: Pull Request #5 — Mozilla Firefox',
          activity_type: 'active',
          source: 'x11',
          event_count: 45,
          browser_context: {
            browser: 'firefox',
            page_title: 'GitHub - tendly/tendly: Pull Request #5',
            url: null,
            domain: 'github.com',
          },
        },
      ],
      app_breakdown: [{ app: 'firefox', duration_ms: 3600000 }],
    });

    const { findByText } = render(Timeline, { currentActivity: null });
    const inspectBtn = await findByText('Inspect [+]');
    await fireEvent.click(inspectBtn);

    // Verify extracted page title and domain badge are rendered
    expect(await findByText('GitHub - tendly/tendly: Pull Request #5')).toBeTruthy();
    expect(await findByText('github.com')).toBeTruthy();
  });
});
