import { describe, expect, it } from 'vitest';

import { createImportPlanner } from '../ImportPlanner.js';
import type { DetectedOpeningLine } from '../../../types/repertoire.js';

const whiteItalian: DetectedOpeningLine = {
  opening: 'Italian Game',
  color: 'White',
  moves: ['e4', 'e5', 'Nf3', 'Nc6', 'Bc4', 'Bc5'],
  display: '1.e4 e5 2.Nf3 Nc6 3.Bc4 Bc5',
};

const blackSicilian: DetectedOpeningLine = {
  opening: 'Sicilian Defence',
  color: 'Black',
  moves: ['e4', 'c5', 'Nf3', 'd6'],
  display: '1.e4 c5 2.Nf3 d6',
};

describe('createImportPlanner', () => {
  it('schedules new lines with deterministic ids and sequential dates', () => {
    const ids = ['line-1', 'line-2'];
    const planner = createImportPlanner({
      clock: () => new Date('2024-03-10T00:00:00Z'),
      idFactory: () => ids.shift() ?? 'unexpected-id',
    });

    const firstPlan = planner.planLine(whiteItalian);
    expect(firstPlan.line.id).toBe('line-1');
    expect(firstPlan.line.scheduledFor).toBe('2024-03-11');
    expect(firstPlan.createdAt.toISOString()).toBe('2024-03-10T00:00:00.000Z');
    expect(firstPlan.messages).toEqual(['Line scheduled for 2024-03-11']);

    const secondPlan = planner.planLine(blackSicilian);
    expect(secondPlan.line.id).toBe('line-2');
    expect(secondPlan.line.scheduledFor).toBe('2024-03-12');
    expect(secondPlan.messages).toEqual(['Line scheduled for 2024-03-12']);
  });

  it('deduplicates lines and reports the existing schedule', () => {
    const planner = createImportPlanner({
      clock: () => new Date('2024-04-01T00:00:00Z'),
      idFactory: () => 'duplicate-id',
    });

    const firstPlan = planner.planLine(whiteItalian);
    const duplicatePlan = planner.planLine({ ...whiteItalian });

    expect(duplicatePlan.line).toBe(firstPlan.line);
    expect(duplicatePlan.messages).toEqual([
      `Line already scheduled for ${firstPlan.line.scheduledFor}`,
    ]);
    expect(duplicatePlan.createdAt.toISOString()).toBe('2024-04-01T00:00:00.000Z');
  });

  it('plans bulk imports against a shared reference date and persists through the adapter', async () => {
    const persisted: string[] = [];
    const planner = createImportPlanner({
      clock: () => new Date('2024-01-01T00:00:00Z'),
      idFactory: () => 'bulk-id',
      persistPlan: async (plan) => {
        persisted.push(`${plan.line.id}:${plan.line.scheduledFor}`);
      },
    });

    const referenceDate = new Date('2024-05-05T09:00:00Z');
    const [firstPlan, secondPlan] = planner.planBulk([
      whiteItalian,
      { ...whiteItalian, moves: ['e4', 'e5', 'Nf3', 'Nc6', 'Bc4', 'Bb5'] },
    ], referenceDate);

    expect(firstPlan.createdAt.toISOString()).toBe('2024-05-05T09:00:00.000Z');
    expect(firstPlan.line.scheduledFor).toBe('2024-05-06');
    expect(secondPlan.line.scheduledFor).toBe('2024-05-07');

    await planner.persist(firstPlan);
    await planner.persist(secondPlan);

    expect(persisted).toEqual([
      `${firstPlan.line.id}:${firstPlan.line.scheduledFor}`,
      `${secondPlan.line.id}:${secondPlan.line.scheduledFor}`,
    ]);
  });
});
