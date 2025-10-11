import { afterEach, describe, expect, it, vi } from 'vitest';

import { readFileText } from '../readFileText';

describe('readFileText', () => {
  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
  });

  it('uses File.text when available', async () => {
    const text = vi.fn(() => Promise.resolve('1.e4 e5 2.d4'));
    const file = { text } as unknown as File;

    await expect(readFileText(file)).resolves.toBe('1.e4 e5 2.d4');
    expect(text).toHaveBeenCalledTimes(1);
  });

  it('falls back to FileReader when File.text is unavailable', async () => {
    const readAsText = vi.fn(function (
      this: { onload: (() => void) | null; result: string },
      file: Blob,
    ) {
      void file;
      this.result = '1.e4 e5 2.d4 exd4 3.c3';
      this.onload?.();
    });

    vi.stubGlobal(
      'FileReader',
      vi.fn(
        () =>
          ({
            onload: null,
            onerror: null,
            readAsText,
            result: '',
          }) as unknown as FileReader,
      ) as unknown as typeof FileReader,
    );

    const file = {} as File;
    await expect(readFileText(file)).resolves.toBe('1.e4 e5 2.d4 exd4 3.c3');
    expect(readAsText).toHaveBeenCalledTimes(1);
  });

  it('rejects when FileReader fails to load', async () => {
    const error = new Error('cannot read');
    vi.stubGlobal(
      'FileReader',
      vi.fn(
        () =>
          ({
            onload: null,
            onerror: null,
            readAsText: function (this: { onerror: (() => void) | null }) {
              this.onerror?.();
            },
            result: null,
            error,
          }) as unknown as FileReader,
      ) as unknown as typeof FileReader,
    );

    const file = {} as File;
    await expect(readFileText(file)).rejects.toBe(error);
  });

  it('treats non-string FileReader results as empty strings', async () => {
    vi.stubGlobal(
      'FileReader',
      vi.fn(
        () =>
          ({
            onload: null,
            onerror: null,
            readAsText: function (this: {
              onload: (() => void) | null;
              result: ArrayBuffer | null;
            }) {
              this.result = new ArrayBuffer(4);
              this.onload?.();
            },
            result: null,
          }) as unknown as FileReader,
      ) as unknown as typeof FileReader,
    );

    const file = {} as File;
    await expect(readFileText(file)).resolves.toBe('');
  });
});
