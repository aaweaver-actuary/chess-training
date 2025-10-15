import { describe, expect, it, expectTypeOf } from 'vitest';
import { applicationControllers, applicationServices, applicationViewModels } from '../index.js';
import type {
  CommandPaletteService,
  DashboardViewModel,
  ImportPlanner,
  OpeningReviewController,
  PgnImportService,
  SessionController,
} from '../index.js';

describe('application layer scaffolding', () => {
  it('exposes planned service contracts', () => {
    expect(applicationServices).toBeDefined();
    expect(Object.keys(applicationServices)).toEqual(
      expect.arrayContaining(['pgnImport', 'commandPalette', 'importPlanner']),
    );
    expectTypeOf<PgnImportService>().toBeObject();
    expectTypeOf<CommandPaletteService>().toBeObject();
    expectTypeOf<ImportPlanner>().toBeObject();
  });

  it('exposes planned controller contracts', () => {
    expect(applicationControllers).toBeDefined();
    expect(Object.keys(applicationControllers)).toEqual(
      expect.arrayContaining(['openingReview', 'session']),
    );
    expectTypeOf<OpeningReviewController>().toBeObject();
    expectTypeOf<SessionController>().toBeObject();
  });

  it('exposes planned view model contracts', () => {
    expect(applicationViewModels).toBeDefined();
    expect(Object.keys(applicationViewModels)).toEqual(expect.arrayContaining(['dashboard']));
    expectTypeOf<DashboardViewModel>().toBeObject();
  });
});
