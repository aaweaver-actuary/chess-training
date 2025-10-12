import { describe, expect, it, vi } from 'vitest';

import type { DetectedOpeningLine } from '../../../types/repertoire';
import { DefaultImportPlanner } from '../ImportPlanner.js';

describe('DefaultImportPlanner', () => {
  const sampleLine: DetectedOpeningLine = {
    opening: 'Danish Gambit',
    color: 'White',
    moves: ['e4', 'e5', 'd4', 'exd4', 'c3'],
    display: '1.e4 e5 2.d4 exd4 3.c3',
  };

  const buildPlanner = (
    overrides: Partial<ConstructorParameters<typeof DefaultImportPlanner>[0]> = {},
  ) => {
    const clockDate = new Date('2024-05-01T08:00:00.000Z');
    let nextId = 1;
    return new DefaultImportPlanner({
      clock: () => clockDate,
      idGenerator: () => `line-${nextId++}`,
      ...overrides,
    });
  };

  it('schedules a new line for the next available day', () => {
    const planner = buildPlanner();

    const plan = planner.planLine(sampleLine);

    expect(plan.line.id).toBe('line-1');
    expect(plan.line.scheduledFor).toBe('2024-05-02');
    expect(plan.line.opening).toBe(sampleLine.opening);
    expect(plan.createdAt.toISOString()).toBe('2024-05-01T08:00:00.000Z');
    expect(plan.messages[0]).toContain('Scheduled');
  });

  it('returns the existing schedule when the line is already planned', () => {
    const planner = buildPlanner();

    const firstPlan = planner.planLine(sampleLine);
    const secondPlan = planner.planLine(sampleLine);

    expect(secondPlan.line).toEqual(firstPlan.line);
    expect(secondPlan.messages[0]).toContain('Already scheduled');
  });

  it('increments the schedule date for each additional line', () => {
    const planner = buildPlanner();

    const extraLine: DetectedOpeningLine = {
      opening: "King's Knight Opening",
      color: 'White',
      moves: ['e4', 'e5', 'Nf3'],
      display: '1.e4 e5 2.Nf3',
    };

    const [firstPlan, secondPlan] = planner.planBulk([sampleLine, extraLine]);

    expect(firstPlan.line.id).toBe('line-1');
    expect(firstPlan.line.scheduledFor).toBe('2024-05-02');
    expect(secondPlan.line.id).toBe('line-2');
    expect(secondPlan.line.scheduledFor).toBe('2024-05-03');
  });

  it('persists a plan using the configured persistence handler', async () => {
    const persist = vi.fn().mockResolvedValue(undefined);
    const planner = buildPlanner({ persistLine: persist });

    const plan = planner.planLine(sampleLine);
    await planner.persist(plan);

    expect(persist).toHaveBeenCalledWith(plan.line);
  });
});
