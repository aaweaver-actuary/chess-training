import { describe, expect, it, vi } from 'vitest';

import type { DetectedOpeningLine } from '../../types/repertoire';

import { createOpeningLineScheduler, linesMatch } from '../importedLines';

describe('importedLines utilities', () => {
  const line: DetectedOpeningLine = {
    opening: "King's Gambit",
    color: 'White',
    moves: ['e4', 'e5', 'f4'],
    display: '1.e4 e5 2.f4',
  };

  it('schedules opening lines with sequential offsets', () => {
    vi.useFakeTimers();
    const now = new Date('2024-03-01T00:00:00Z');
    vi.setSystemTime(now);

    const scheduler = createOpeningLineScheduler();

    const first = scheduler(line, 0);
    const second = scheduler(line, 1);

    expect(first.scheduledFor).toBe('2024-03-02');
    expect(second.scheduledFor).toBe('2024-03-03');
    expect(first.id).not.toBe(second.id);
    expect(first.moves).toEqual(line.moves);

    vi.useRealTimers();
  });

  it('compares scheduled and detected lines for equality', () => {
    const scheduler = createOpeningLineScheduler();
    const scheduled = scheduler(line, 2);

    expect(linesMatch(scheduled, line)).toBe(true);
    expect(
      linesMatch(
        {
          ...scheduled,
          moves: [...scheduled.moves.slice(0, -1), 'Nc6'],
        },
        line,
      ),
    ).toBe(false);
  });

  it('detects when openings differ', () => {
    const scheduler = createOpeningLineScheduler();
    const scheduled = scheduler(line, 0);

    expect(
      linesMatch(
        {
          ...scheduled,
          opening: 'Vienna Game',
        },
        line,
      ),
    ).toBe(false);
  });

  it('detects when move counts differ', () => {
    const scheduler = createOpeningLineScheduler();
    const scheduled = scheduler(line, 0);

    expect(
      linesMatch(
        {
          ...scheduled,
          moves: scheduled.moves.slice(0, -1),
        },
        line,
      ),
    ).toBe(false);
  });

  it('generates unique IDs for lines scheduled at the same time', () => {
    // Use a fixed clock to ensure multiple lines are created at exactly the same millisecond
    const fixedClock = () => new Date('2024-03-01T00:00:00Z');
    const scheduler = createOpeningLineScheduler(fixedClock);

    // Create multiple lines with the same offset at the same time
    // With the old implementation using timestamp + offset, these would have the same ID
    const first = scheduler(line, 0);
    const secondLine: DetectedOpeningLine = {
      opening: "Queen's Gambit",
      color: 'White',
      moves: ['d4', 'd5', 'c4'],
      display: '1.d4 d5 2.c4',
    };
    const second = scheduler(secondLine, 0);

    // IDs should be unique even with same timestamp and offset
    expect(first.id).not.toBe(second.id);
  });
});
