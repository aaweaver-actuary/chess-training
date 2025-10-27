import { describe, expect, it, vi } from 'vitest';

import {
  type PgnImportOutcome,
  type PgnImportService,
  type PgnImportSource,
} from '../PgnImportService.js';
import { createPgnImportService } from '../createPgnImportService.js';

type ServiceFactoryOptions = Parameters<typeof createPgnImportService>[0];

const fixedNow = new Date('2025-01-20T00:00:00.000Z');
const createService = (options: ServiceFactoryOptions = {}): PgnImportService => {
  return createPgnImportService({
    clock: () => fixedNow,
    generateId: () => 'message-1',
    ...options,
  });
};

describe('createPgnImportService', () => {
  const detect = async (
    service: PgnImportService,
    source: PgnImportSource,
  ): Promise<PgnImportOutcome> => {
    const outcome = await service.detect(source);
    expect(outcome.preview).toBeDefined();
    return outcome;
  };

  it('normalizes PGN text and detects the Danish Gambit', async () => {
    const service = createService();
    const source: PgnImportSource = {
      kind: 'text',
      value: ' 1. e4 e5 2. d4   exd4 3.c3 ',
    };

    const outcome = await detect(service, source);

    expect(outcome.preview.normalizedPgn).toEqual('1.e4 e5 2.d4 exd4 3.c3');
    expect(outcome.preview.detectedLines).toEqual([
      {
        opening: 'Danish Gambit',
        color: 'White',
        moves: ['e4', 'e5', 'd4', 'exd4', 'c3'],
        display: '1.e4 e5 2.d4 exd4 3.c3',
      },
    ]);
    expect(outcome.preview.scheduledLines).toEqual([]);
    expect(outcome.errors).toEqual([]);
    expect(outcome.messages).toEqual([
      {
        id: 'message-1',
        tone: 'info',
        headline: 'Detected Danish Gambit (White)',
        body: 'Line preview: 1.e4 e5 2.d4 exd4 3.c3',
        dispatchAt: fixedNow,
      },
    ]);
  });

  it('reads PGN lines from Blob sources', async () => {
    const service = createService();
    const blob = {
      text: vi.fn().mockResolvedValue('1.e4 e5 2.Nf3 Nc6'),
    } as unknown as Blob;
    const source: PgnImportSource = { kind: 'file', value: blob };

    const outcome = await detect(service, source);

    expect(outcome.preview.detectedLines).toEqual([
      {
        opening: "King's Knight Opening",
        color: 'White',
        moves: ['e4', 'e5', 'Nf3', 'Nc6'],
        display: '1.e4 e5 2.Nf3 Nc6',
      },
    ]);
    expect(blob.text).toHaveBeenCalledTimes(1);
    expect(outcome.messages[0]).toMatchObject({
      id: 'message-1',
      tone: 'info',
      headline: "Detected King's Knight Opening (White)",
    });
  });

  it('reports an error when no known opening matches', async () => {
    const service = createService();
    const source: PgnImportSource = {
      kind: 'text',
      value: '1.d4 d5 2.c4 e6',
    };

    const outcome = await detect(service, source);

    expect(outcome.preview.detectedLines).toEqual([]);
    expect(outcome.errors).toEqual([
      'No matching opening found for the provided PGN.',
    ]);
    expect(outcome.messages).toEqual([]);
  });

  it('reports an error when the PGN text does not contain moves', async () => {
    const service = createService();
    const outcome = await detect(service, { kind: 'text', value: '   ' });

    expect(outcome.preview.normalizedPgn).toEqual('');
    expect(outcome.preview.detectedLines).toEqual([]);
    expect(outcome.errors).toEqual(['PGN source did not include any moves.']);
  });
});
