import { act, renderHook } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { CommandPaletteService } from '../../services/CommandPaletteService.js';
import type { PgnImportOutcome, PgnImportService } from '../../services/PgnImportService.js';
import { usePgnImport } from '../usePgnImport.js';

const sampleOutcome: PgnImportOutcome = {
  preview: {
    normalizedPgn: '1.e4 e5 2.Nf3 Nc6',
    detectedLines: [
      {
        opening: "King's Knight Opening",
        color: 'White',
        moves: ['e4', 'e5', 'Nf3', 'Nc6'],
        display: '1.e4 e5 2.Nf3 Nc6',
      },
    ],
    scheduledLines: [
      {
        opening: "King's Knight Opening",
        color: 'White',
        moves: ['e4', 'e5', 'Nf3', 'Nc6'],
        display: '1.e4 e5 2.Nf3 Nc6',
        id: 'line-1',
        scheduledFor: '2024-01-05',
      },
    ],
  },
  messages: [
    {
      id: 'message-1',
      tone: 'success',
      headline: 'Scheduled for 2024-01-05 in your white repertoire.',
      dispatchAt: new Date('2024-01-02T00:00:00Z'),
    },
  ],
  errors: [],
};

const createService = (overrides: Partial<PgnImportService> = {}): PgnImportService => ({
  detect: vi.fn().mockResolvedValue(sampleOutcome),
  acknowledge: vi.fn(),
  clear: vi.fn(),
  ...overrides,
});

const createCommandPalette = (): CommandPaletteService => ({
  register: vi.fn(),
  unregister: vi.fn(),
  list: vi.fn(),
  execute: vi.fn(),
  subscribe: vi.fn(),
  reset: vi.fn(),
});

describe('usePgnImport', () => {
  beforeEach(() => {
    vi.useRealTimers();
  });

  it('detects openings from text input and exposes the preview state', async () => {
    const service = createService();
    const { result } = renderHook(() => usePgnImport({ service }));

    act(() => {
      result.current.actions.activate('paste');
    });

    await act(async () => {
      await result.current.actions.setSourceText('1.e4 e5 2.Nf3 Nc6');
    });

    expect(service.detect).toHaveBeenCalledWith({ kind: 'text', value: '1.e4 e5 2.Nf3 Nc6' });
    expect(result.current.state.preview).toEqual(sampleOutcome.preview);
    expect(result.current.state.messages).toEqual(sampleOutcome.messages);
    expect(result.current.state.errors).toEqual([]);
  });

  it('acknowledges feedback messages and clears them from state', async () => {
    const service = createService();
    const { result } = renderHook(() => usePgnImport({ service }));

    act(() => {
      result.current.actions.activate('paste');
    });

    await act(async () => {
      await result.current.actions.setSourceText('1.e4 e5 2.Nf3 Nc6');
    });

    act(() => {
      result.current.actions.acknowledgeMessages();
    });

    expect(service.acknowledge).toHaveBeenCalledWith(sampleOutcome);
    expect(result.current.state.messages).toEqual([]);
  });

  it('registers a collapse command with the command palette', async () => {
    const service = createService();
    const commandPalette = createCommandPalette();

    const { result, unmount } = renderHook(() => usePgnImport({ service, commandPalette }));

    expect(commandPalette.register).toHaveBeenCalledTimes(1);
    const [registration, handler] = commandPalette.register.mock.calls[0];
    expect(registration.id).toBe('pgn-import:collapse');

    act(() => {
      result.current.actions.activate('paste');
    });

    await act(async () => {
      await result.current.actions.setSourceText('1.e4 e5 2.Nf3 Nc6');
    });

    await act(async () => {
      await handler({ source: 'palette', issuedAt: new Date('2024-01-02T00:00:00Z') });
    });

    expect(result.current.state.isExpanded).toBe(false);

    unmount();
    expect(commandPalette.unregister).toHaveBeenCalledWith('pgn-import:collapse');
  });

  it('collapses the pane and clears internal state', async () => {
    const service = createService();
    const { result } = renderHook(() => usePgnImport({ service }));

    act(() => {
      result.current.actions.activate('paste');
    });

    await act(async () => {
      await result.current.actions.setSourceText('1.e4 e5 2.Nf3 Nc6');
    });

    act(() => {
      result.current.actions.collapse();
    });

    expect(service.clear).toHaveBeenCalled();
    expect(result.current.state).toMatchObject({
      isExpanded: false,
      mode: 'idle',
      sourceText: '',
      messages: [],
      errors: [],
    });
    expect(result.current.state.preview).toBeUndefined();
  });

  it('handles PGN files by delegating to the service detect method', async () => {
    const detect = vi.fn().mockResolvedValue(sampleOutcome);
    const service = createService({ detect });
    const { result } = renderHook(() => usePgnImport({ service }));

    const file = new Blob(['1.e4 e5 2.Nf3 Nc6']);

    await act(async () => {
      await result.current.actions.importFromFile(file);
    });

    expect(detect).toHaveBeenCalledWith({ kind: 'file', value: file });
    expect(result.current.state.mode).toBe('upload');
    expect(result.current.state.preview).toEqual(sampleOutcome.preview);
  });
});
