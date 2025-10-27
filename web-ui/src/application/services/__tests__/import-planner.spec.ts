import { describe, expect, it, vi } from 'vitest';

import type { ImportPlan } from '../ImportPlanner.js';
import type { DetectedOpeningLine, ScheduledOpeningLine } from '../../../types/repertoire.js';
import { DefaultImportPlanner } from '../DefaultImportPlanner.js';

const makeLine = (overrides: Partial<DetectedOpeningLine> = {}): DetectedOpeningLine => ({
  opening: 'London System',
  color: 'White',
  moves: ['d4', 'Nf3', 'Bf4'],
  display: '1.d4 Nf3 2.Bf4',
  ...overrides,
});

type PlannerOptions = {
  now?: Date | (() => Date);
  ids?: string[];
  existing?: ScheduledOpeningLine[];
  persist?: (plan: ImportPlan) => Promise<void> | void;
};

const buildPlanner = (options: PlannerOptions = {}): DefaultImportPlanner => {
  const { now, ids = ['id-1', 'id-2', 'id-3'], existing = [], persist } = options;
  const clock = typeof now === 'function' ? now : () => new Date(now ?? '2024-01-01T00:00:00.000Z');
  const idGenerator = vi.fn(() => {
    const value = ids.shift();
    if (!value) {
      throw new Error('Exhausted test UUIDs');
    }

    return value;
  });

  return new DefaultImportPlanner({ clock, generateId: idGenerator, initialLines: existing, persist });
};

describe('DefaultImportPlanner', () => {
  it('plans a single line for the day after the reference date', () => {
    const planner = buildPlanner({ now: new Date('2024-04-10T12:00:00.000Z'), ids: ['scheduled-1'] });
    const line = makeLine();

    const plan = planner.planLine(line, new Date('2024-04-01T00:00:00.000Z'));

    expect(plan.line).toEqual({
      ...line,
      id: 'scheduled-1',
      scheduledFor: '2024-04-02',
    });
    expect(plan.createdAt.toISOString()).toBe('2024-04-10T12:00:00.000Z');
    expect(plan.messages).toEqual([
      'Scheduled London System (White) for 2024-04-02',
    ]);
  });

  it('increments the scheduled date for each newly planned line before persisting', () => {
    const planner = buildPlanner({ now: () => new Date('2024-04-10T12:00:00.000Z') });
    const first = planner.planLine(makeLine({ opening: 'London System' }), new Date('2024-04-01T00:00:00.000Z'));
    const second = planner.planLine(
      makeLine({ opening: 'Jobava London', moves: ['d4', 'Nc3', 'Bf4'], display: '1.d4 Nc3 2.Bf4' }),
      new Date('2024-04-01T00:00:00.000Z'),
    );

    expect(first.line.scheduledFor).toBe('2024-04-02');
    expect(second.line.scheduledFor).toBe('2024-04-03');
    expect(second.messages).toEqual([
      'Scheduled Jobava London (White) for 2024-04-03',
    ]);
  });

  it('returns the existing schedule when the line has already been persisted', () => {
    const existingLine: ScheduledOpeningLine = {
      ...makeLine(),
      id: 'existing-1',
      scheduledFor: '2024-04-05',
    };
    const planner = buildPlanner({ existing: [existingLine] });

    const plan = planner.planLine(makeLine(), new Date('2024-04-01T00:00:00.000Z'));

    expect(plan.line).toEqual(existingLine);
    expect(plan.messages).toEqual([
      'Line already scheduled for 2024-04-05',
    ]);
  });

  it('persists new plans using the provided sink and tracks them as existing', async () => {
    const save = vi.fn(async () => {});
    const planner = buildPlanner({ now: new Date('2024-04-10T12:00:00.000Z'), persist: save });
    const plan = planner.planLine(makeLine(), new Date('2024-04-01T00:00:00.000Z'));

    await planner.persist(plan);

    expect(save).toHaveBeenCalledWith(plan);

    const duplicate = planner.planLine(makeLine(), new Date('2024-04-01T00:00:00.000Z'));
    expect(duplicate.line).toEqual(plan.line);
    expect(duplicate.messages).toEqual([
      'Line already scheduled for 2024-04-02',
    ]);
  });
});
