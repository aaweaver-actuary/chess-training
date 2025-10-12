import { describe, expect, it, beforeEach, vi } from 'vitest';

import type { ImportPlan, ImportPlanner } from '../ImportPlanner.js';
import { createPgnImportService } from '../PgnImportService.js';

const buildPlanner = (plans: ImportPlan[]): ImportPlanner => {
  return {
    planLine: vi.fn(() => {
      if (plans.length === 0) {
        throw new Error('no plans available');
      }
      return plans[0];
    }),
    planBulk: vi.fn(() => plans),
    persist: vi.fn(async () => {
      // no-op for tests
    }),
  } satisfies ImportPlanner;
};

describe('createPgnImportService', () => {
  const sampleDetectedLine = {
    opening: 'Danish Gambit',
    color: 'White',
    moves: ['e4', 'e5', 'd4', 'exd4', 'c3'],
    display: '1.e4 e5 2.d4 exd4 3.c3',
  };

  const samplePlan: ImportPlan = {
    line: {
      ...sampleDetectedLine,
      id: 'line-1',
      scheduledFor: '2024-01-05',
    },
    createdAt: new Date('2024-01-01T00:00:00Z'),
    messages: ['Scheduled for 2024-01-05 in your white Danish Gambit repertoire.'],
  };

  const detectionSource = '1. e4 e5 2. d4 exd4 3. c3';

  const factory = () =>
    createPgnImportService({
      importPlanner: buildPlanner([samplePlan]),
      idFactory: () => 'message-1',
      clock: () => new Date('2024-01-02T00:00:00Z'),
    });

  beforeEach(() => {
    vi.useRealTimers();
  });

  it('detects a supported opening from text sources and schedules planner messages', async () => {
    const service = factory();

    const outcome = await service.detect({ kind: 'text', value: detectionSource });

    expect(outcome.preview.normalizedPgn).toEqual('1.e4 e5 2.d4 exd4 3.c3');
    expect(outcome.preview.detectedLines).toEqual([sampleDetectedLine]);
    expect(outcome.preview.scheduledLines).toEqual([samplePlan.line]);
    expect(outcome.messages).toEqual([
      {
        id: 'message-1',
        tone: 'success',
        headline: samplePlan.messages[0],
        body: undefined,
        dispatchAt: new Date('2024-01-02T00:00:00Z'),
      },
    ]);
    expect(outcome.errors).toEqual([]);
  });

  it('only emits planner messages once they are acknowledged', async () => {
    const service = factory();

    const firstOutcome = await service.detect({ kind: 'text', value: detectionSource });
    service.acknowledge(firstOutcome);

    const secondOutcome = await service.detect({ kind: 'text', value: detectionSource });

    expect(secondOutcome.messages).toEqual([]);

    service.clear();

    const thirdOutcome = await service.detect({ kind: 'text', value: detectionSource });
    expect(thirdOutcome.messages).toHaveLength(1);
  });

  it('returns meaningful errors when no opening is detected', async () => {
    const service = factory();

    const outcome = await service.detect({ kind: 'text', value: '1. d4 Nf6 2. c4 g6' });

    expect(outcome.preview.detectedLines).toEqual([]);
    expect(outcome.preview.scheduledLines).toEqual([]);
    expect(outcome.messages).toEqual([]);
    expect(outcome.errors).toEqual([
      "We could not recognize that PGN yet. Try a standard Danish Gambit or King's Knight Opening line.",
    ]);
  });

  it('normalizes text extracted from a PGN file source', async () => {
    const textBlob = new Blob([detectionSource]);
    const readText = vi.fn(async () => detectionSource);
    const service = createPgnImportService({
      importPlanner: buildPlanner([samplePlan]),
      idFactory: () => 'message-1',
      clock: () => new Date('2024-01-02T00:00:00Z'),
      readText,
    });

    const outcome = await service.detect({ kind: 'file', value: textBlob });

    expect(readText).toHaveBeenCalledWith(textBlob);
    expect(outcome.preview.detectedLines).toHaveLength(1);
    expect(outcome.errors).toEqual([]);
  });

  it('reports errors when a PGN file cannot be read', async () => {
    const service = createPgnImportService({
      importPlanner: buildPlanner([]),
      idFactory: () => 'message-1',
      clock: () => new Date('2024-01-02T00:00:00Z'),
      readText: async () => {
        throw new Error('boom');
      },
    });

    const outcome = await service.detect({ kind: 'file', value: new Blob([]) });

    expect(outcome.preview.detectedLines).toEqual([]);
    expect(outcome.errors).toEqual(['We could not read that PGN file. Please try again.']);
  });
});
