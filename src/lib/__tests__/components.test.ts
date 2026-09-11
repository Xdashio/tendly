import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import Badge from '../components/Badge.svelte';
import EmptyState from '../components/EmptyState.svelte';

describe('Frontend Component Foundation', () => {
  it('renders Badge with default variant', () => {
    const { getByText } = render(Badge, { text: 'Active' });
    const badge = getByText('Active');
    expect(badge).toBeTruthy();
    expect(badge.className).toContain('font-mono');
  });

  it('renders EmptyState with title and description', () => {
    const { getByText } = render(EmptyState, {
      title: 'No Items Found',
      description: 'Test description content',
    });
    expect(getByText('No Items Found')).toBeTruthy();
    expect(getByText('Test description content')).toBeTruthy();
  });
});
