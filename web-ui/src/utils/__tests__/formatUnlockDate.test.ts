import { describe, expect, it } from 'vitest';

import { formatUnlockDate, UNLOCK_DATE_FALLBACK_LABEL } from '../formatUnlockDate';

describe('formatUnlockDate', () => {
  it('formats a valid ISO string using the current locale short month and day', () => {
    const formatted = formatUnlockDate('2024-01-20');

    expect(formatted).toMatch(/Jan\s+20|20\s+Jan/);
  });

  it('returns a fallback label when the input cannot be parsed as a date', () => {
    expect(formatUnlockDate('not-a-date')).toBe(UNLOCK_DATE_FALLBACK_LABEL);
  });
});
