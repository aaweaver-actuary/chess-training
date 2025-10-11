import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type { CommandDispatcher } from '../../utils/commandDispatcher';
import type { ImportResult } from '../../types/repertoire';
import { PgnImportPane } from '../PgnImportPane';

describe('PgnImportPane', () => {
  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
  });

  it('expands when the pointer enters the pane container', () => {
    render(
      <PgnImportPane
        onImportLine={() => ({
          added: false,
          line: {
            id: 'test',
            opening: 'Danish Gambit',
            color: 'White',
            moves: [],
            display: '',
            scheduledFor: new Date().toISOString(),
          },
        })}
      />,
    );

    const pane = screen.getByRole('complementary', { name: /pgn import tools/i });
    const handle = screen.getByRole('button', { name: /open pgn import tools/i });
    expect(handle).toHaveAttribute('aria-expanded', 'false');

    fireEvent.pointerEnter(pane);

    expect(handle).toHaveAttribute('aria-expanded', 'true');
  });

  it('remains expanded after the pointer leaves', () => {
    render(
      <PgnImportPane
        onImportLine={() => ({
          added: false,
          line: {
            id: 'test',
            opening: 'Danish Gambit',
            color: 'White',
            moves: [],
            display: '',
            scheduledFor: new Date().toISOString(),
          },
        })}
      />,
    );

    const pane = screen.getByRole('complementary', { name: /pgn import tools/i });
    const handle = screen.getByRole('button', { name: /open pgn import tools/i });

    fireEvent.pointerEnter(pane);
    expect(handle).toHaveAttribute('aria-expanded', 'true');

    fireEvent.pointerLeave(pane);

    expect(handle).toHaveAttribute('aria-expanded', 'true');
  });

  it('collapses when clicking outside of the pane', async () => {
    render(
      <div>
        <PgnImportPane
          onImportLine={() => ({
            added: false,
            line: {
              id: 'test',
              opening: 'Danish Gambit',
              color: 'White',
              moves: [],
              display: '',
              scheduledFor: new Date().toISOString(),
            },
          })}
        />
        <button type="button">Outside</button>
      </div>,
    );

    const pane = screen.getByRole('complementary', { name: /pgn import tools/i });
    const handle = screen.getByRole('button', { name: /open pgn import tools/i });
    expect(handle).toHaveAttribute('aria-expanded', 'false');

    fireEvent.pointerEnter(pane);
    expect(handle).toHaveAttribute('aria-expanded', 'true');

    const outsideButton = screen.getByRole('button', { name: 'Outside' });
    fireEvent.pointerDown(outsideButton);
    fireEvent.click(outsideButton);

    await waitFor(() => {
      expect(handle).toHaveAttribute('aria-expanded', 'false');
    });
  });

  it('does not collapse when focus moves within the pane', async () => {
    const user = userEvent.setup();

    render(
      <PgnImportPane
        onImportLine={() => ({
          added: true,
          line: {
            id: 'import-test',
            opening: 'Danish Gambit',
            color: 'White',
            moves: ['e4', 'e5', 'd4', 'exd4', 'c3'],
            display: '1.e4 e5 2.d4 exd4 3.c3',
            scheduledFor: new Date().toISOString(),
          },
        })}
      />,
    );

    const handle = screen.getByRole('button', {
      name: /open pgn import tools/i,
    });
    fireEvent.pointerEnter(handle);

    const pasteOption = await screen.findByRole('button', {
      name: /paste pgn/i,
    });
    await user.click(pasteOption);

    const textarea = await screen.findByLabelText(/pgn move input/i);
    await user.type(textarea, '1.e4 e5 2.d4 exd4 3.c3');

    const confirm = await screen.findByRole('button', {
      name: /add to danish gambit \(white\)/i,
    });

    textarea.focus();
    fireEvent.blur(textarea, { relatedTarget: confirm });

    expect(handle).toHaveAttribute('aria-expanded', 'true');
  });

  it('collapses when focus leaves the pane entirely', async () => {
    const user = userEvent.setup();

    render(
      <div>
        <PgnImportPane
          onImportLine={() => ({
            added: true,
            line: {
              id: 'import-test',
              opening: 'Danish Gambit',
              color: 'White',
              moves: ['e4', 'e5', 'd4', 'exd4', 'c3'],
              display: '1.e4 e5 2.d4 exd4 3.c3',
              scheduledFor: new Date().toISOString(),
            },
          })}
        />
        <button type="button">Outside</button>
      </div>,
    );

    const handle = screen.getByRole('button', {
      name: /open pgn import tools/i,
    });
    fireEvent.pointerEnter(handle);

    const pasteOption = await screen.findByRole('button', {
      name: /paste pgn/i,
    });
    await user.click(pasteOption);

    const textarea = await screen.findByLabelText(/pgn move input/i);
    const outsideButton = screen.getByRole('button', { name: 'Outside' });

    textarea.focus();
    fireEvent.blur(textarea, { relatedTarget: outsideButton });
    outsideButton.focus();

    await waitFor(() => {
      expect(handle).toHaveAttribute('aria-expanded', 'false');
    });
  });

  it('keeps the upload tools open when the file input loses focus without a next target', async () => {
    const user = userEvent.setup();

    render(
      <PgnImportPane
        onImportLine={() => ({
          added: false,
          line: {
            id: 'focus-upload',
            opening: 'Danish Gambit',
            color: 'White',
            moves: [],
            display: '',
            scheduledFor: new Date().toISOString(),
          },
        })}
      />,
    );

    const handle = screen.getByRole('button', { name: /open pgn import tools/i });
    fireEvent.pointerEnter(handle);

    const uploadOption = await screen.findByRole('button', { name: /upload pgn/i });
    await user.click(uploadOption);

    const fileInput = await screen.findByLabelText(/upload pgn file/i);
    fileInput.focus();
    fireEvent.blur(fileInput, { relatedTarget: null });

    expect(handle).toHaveAttribute('aria-expanded', 'true');
  });

  it('keeps the upload tools open when focus moves within the upload form', async () => {
    const user = userEvent.setup();

    render(
      <PgnImportPane
        onImportLine={() => ({
          added: false,
          line: {
            id: 'upload-focus-internal',
            opening: 'Danish Gambit',
            color: 'White',
            moves: [],
            display: '',
            scheduledFor: new Date().toISOString(),
          },
        })}
      />,
    );

    const handle = screen.getByRole('button', { name: /open pgn import tools/i });
    fireEvent.pointerEnter(handle);

    const uploadOption = await screen.findByRole('button', { name: /upload pgn/i });
    await user.click(uploadOption);

    const fileInput = await screen.findByLabelText(/upload pgn file/i);
    const textarea = await screen.findByLabelText(/pgn move input/i);

    fileInput.focus();
    fireEvent.blur(fileInput, { relatedTarget: textarea });

    expect(handle).toHaveAttribute('aria-expanded', 'true');
  });

  it('keeps the upload tools open when the file input blurs to an external element', async () => {
    const user = userEvent.setup();

    render(
      <div>
        <PgnImportPane
          onImportLine={() => ({
            added: false,
            line: {
              id: 'upload-focus-exit',
              opening: 'Danish Gambit',
              color: 'White',
              moves: [],
              display: '',
              scheduledFor: new Date().toISOString(),
            },
          })}
        />
        <button type="button">Outside</button>
      </div>,
    );

    const handle = screen.getByRole('button', { name: /open pgn import tools/i });
    fireEvent.pointerEnter(handle);

    const uploadOption = await screen.findByRole('button', { name: /upload pgn/i });
    await user.click(uploadOption);

    const fileInput = await screen.findByLabelText(/upload pgn file/i);
    const outsideButton = screen.getByRole('button', { name: 'Outside' });

    fileInput.focus();
    fireEvent.blur(fileInput, { relatedTarget: outsideButton });
    outsideButton.focus();

    expect(handle).toHaveAttribute('aria-expanded', 'true');
  });

  it('resets the detected line when the PGN input becomes empty', async () => {
    const user = userEvent.setup();

    render(
      <PgnImportPane
        onImportLine={() => ({
          added: true,
          line: {
            id: 'line',
            opening: 'Danish Gambit',
            color: 'White',
            moves: ['e4', 'e5', 'd4', 'exd4', 'c3'],
            display: '1.e4 e5 2.d4 exd4 3.c3',
            scheduledFor: new Date().toISOString(),
          },
        })}
      />,
    );

    const handle = screen.getByRole('button', {
      name: /open pgn import tools/i,
    });
    fireEvent.pointerEnter(handle);

    const pasteOption = await screen.findByRole('button', {
      name: /paste pgn/i,
    });
    await user.click(pasteOption);

    const textarea = await screen.findByLabelText(/pgn move input/i);
    await user.type(textarea, '1.e4 e5 2.d4 exd4 3.c3');
    expect(await screen.findByRole('status')).toHaveTextContent(/detected/i);

    await user.clear(textarea);
    await user.type(textarea, '   ');

    expect(screen.queryByRole('status')).not.toBeInTheDocument();
  });

  it('registers a command dispatcher shortcut', () => {
    const register = vi.fn();
    const unregister = vi.fn();

    const dispatcher: CommandDispatcher = {
      register,
      unregister,
      dispatch: vi.fn(),
    };

    const { unmount } = render(
      <PgnImportPane
        onImportLine={() => ({
          added: false,
          line: {
            id: 'test',
            opening: 'Danish Gambit',
            color: 'White',
            moves: [],
            display: '',
            scheduledFor: new Date().toISOString(),
          },
        })}
        commandDispatcher={dispatcher}
      />,
    );

    expect(register).toHaveBeenCalledWith('x', expect.any(Function));

    unmount();
    expect(unregister).toHaveBeenCalledWith('x');
  });

  it('displays fallback scheduling messaging when the unlock date cannot be parsed', async () => {
    const onImportLine = vi.fn(
      (): ImportResult => ({
        added: true,
        line: {
          id: 'import-test',
          opening: 'Danish Gambit',
          color: 'White',
          moves: ['e4', 'e5', 'd4', 'exd4', 'c3'],
          display: '1.e4 e5 2.d4 exd4 3.c3',
          scheduledFor: 'not-a-valid-date',
        },
      }),
    );

    const user = userEvent.setup();

    render(<PgnImportPane onImportLine={onImportLine} />);

    const handle = screen.getByRole('button', { name: /open pgn import tools/i });
    fireEvent.pointerEnter(handle);

    const pasteOption = await screen.findByRole('button', { name: /paste pgn/i });
    await user.click(pasteOption);

    const textarea = await screen.findByLabelText(/pgn move input/i);
    await user.type(textarea, '1.e4 e5 2.d4 exd4 3.c3');

    const confirmButton = await screen.findByRole('button', {
      name: /add to danish gambit \(white\)/i,
    });
    await user.click(confirmButton);

    expect(onImportLine).toHaveBeenCalled();

    const feedback = await screen.findByRole('status');
    expect(feedback).toHaveTextContent('Line added to your white Danish Gambit repertoire.');
  });

  it('notifies when the detected line already exists in the repertoire using fallback messaging', async () => {
    const onImportLine = vi.fn(
      (): ImportResult => ({
        added: false,
        line: {
          id: 'import-existing',
          opening: 'Danish Gambit',
          color: 'White',
          moves: ['e4', 'e5', 'd4', 'exd4', 'c3'],
          display: '1.e4 e5 2.d4 exd4 3.c3',
          scheduledFor: 'not-a-valid-date',
        },
      }),
    );

    const user = userEvent.setup();

    render(<PgnImportPane onImportLine={onImportLine} />);

    const handle = screen.getByRole('button', { name: /open pgn import tools/i });
    fireEvent.pointerEnter(handle);

    const pasteOption = await screen.findByRole('button', { name: /paste pgn/i });
    await user.click(pasteOption);

    const textarea = await screen.findByLabelText(/pgn move input/i);
    await user.type(textarea, '1.e4 e5 2.d4 exd4 3.c3');

    const confirmButton = await screen.findByRole('button', {
      name: /add to danish gambit \(white\)/i,
    });
    await user.click(confirmButton);

    expect(onImportLine).toHaveBeenCalled();

    const feedback = await screen.findByText(
      'This Danish Gambit line is already part of your white repertoire.',
    );
    expect(feedback).toHaveAttribute('role', 'status');
  });

  it("recognizes a King's Knight Opening line from a short PGN import", async () => {
    const onImportLine = vi.fn(
      (): ImportResult => ({
        added: true,
        line: {
          id: 'import-kko',
          opening: "King's Knight Opening",
          color: 'White',
          moves: ['e4', 'e5', 'Nf3'],
          display: '1.e4 e5 2.Nf3',
          scheduledFor: '2024-01-02',
        },
      }),
    );

    const user = userEvent.setup();

    render(<PgnImportPane onImportLine={onImportLine} />);

    const handle = screen.getByRole('button', { name: /open pgn import tools/i });
    fireEvent.pointerEnter(handle);

    const pasteOption = await screen.findByRole('button', { name: /paste pgn/i });
    await user.click(pasteOption);

    const textarea = await screen.findByLabelText(/pgn move input/i);
    await user.type(textarea, '1. e4 e5 2.Nf3');

    const confirmButton = await screen.findByRole('button', {
      name: /add to king's knight opening \(white\)/i,
    });
    await user.click(confirmButton);

    expect(onImportLine).toHaveBeenCalledWith({
      opening: "King's Knight Opening",
      color: 'White',
      moves: ['e4', 'e5', 'Nf3'],
      display: '1.e4 e5 2.Nf3',
    });
  });

  it('offers a PGN upload option alongside paste import', async () => {
    const user = userEvent.setup();

    render(
      <PgnImportPane
        onImportLine={() => ({
          added: false,
          line: {
            id: 'upload-test',
            opening: 'Danish Gambit',
            color: 'White',
            moves: [],
            display: '',
            scheduledFor: new Date().toISOString(),
          },
        })}
      />,
    );

    const handle = screen.getByRole('button', { name: /open pgn import tools/i });
    fireEvent.pointerEnter(handle);

    const uploadOption = await screen.findByRole('button', { name: /upload pgn/i });
    expect(uploadOption).toHaveAttribute('aria-pressed', 'false');

    await user.click(uploadOption);

    const fileInput = await screen.findByLabelText(/upload pgn file/i);
    expect(fileInput).toHaveAttribute('type', 'file');
  });

  it('assigns distinct textarea ids to paste and upload modes', async () => {
    const user = userEvent.setup();

    render(
      <PgnImportPane
        onImportLine={() => ({
          added: false,
          line: {
            id: 'upload-id-test',
            opening: 'Danish Gambit',
            color: 'White',
            moves: [],
            display: '',
            scheduledFor: new Date().toISOString(),
          },
        })}
      />,
    );

    const handle = screen.getByRole('button', { name: /open pgn import tools/i });
    fireEvent.pointerEnter(handle);

    const pasteOption = await screen.findByRole('button', { name: /paste pgn/i });
    await user.click(pasteOption);

    const pasteLabel = await screen.findByText(/paste moves/i);
    const pasteTextarea = await screen.findByLabelText(/pgn move input/i);
    expect(pasteTextarea).toHaveAttribute('id');
    const pasteTextareaId = pasteTextarea.getAttribute('id');
    expect(pasteTextareaId).not.toBeNull();
    expect(pasteLabel).toHaveAttribute('for', pasteTextareaId as string);

    const uploadOption = await screen.findByRole('button', { name: /upload pgn/i });
    await user.click(uploadOption);

    const uploadLabel = await screen.findByText(/review moves/i);
    const uploadTextarea = await screen.findByLabelText(/pgn move input/i);
    expect(uploadTextarea).toHaveAttribute('id');
    const uploadTextareaId = uploadTextarea.getAttribute('id');
    expect(uploadTextareaId).not.toBeNull();
    expect(uploadLabel).toHaveAttribute('for', uploadTextareaId as string);

    expect(uploadTextarea).not.toBe(pasteTextarea);
    expect(uploadTextareaId).not.toEqual(pasteTextareaId);
  });

  it('treats uploaded PGNs the same as pasted PGNs', async () => {
    const user = userEvent.setup();

    render(
      <PgnImportPane
        onImportLine={() => ({
          added: false,
          line: {
            id: 'equivalence-test',
            opening: 'Danish Gambit',
            color: 'White',
            moves: [],
            display: '',
            scheduledFor: new Date().toISOString(),
          },
        })}
      />,
    );

    const handle = screen.getByRole('button', { name: /open pgn import tools/i });
    fireEvent.pointerEnter(handle);

    const pasteOption = await screen.findByRole('button', { name: /paste pgn/i });
    await user.click(pasteOption);

    const textarea = await screen.findByLabelText(/pgn move input/i);
    await user.type(textarea, '1. e4 e5 2.d4');

    const pasteAlert = await screen.findByRole('alert');
    const pasteMessage = pasteAlert.textContent;
    expect(pasteMessage).toMatch(/could not recognize/i);

    await user.clear(textarea);
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();

    const uploadOption = await screen.findByRole('button', { name: /upload pgn/i });
    await user.click(uploadOption);

    const fileInput = await screen.findByLabelText(/upload pgn file/i);
    const file = new File(['1. e4 e5 2.d4'], 'line.pgn', {
      type: 'application/x-chess-pgn',
    });
    await user.upload(fileInput, file);

    const uploadTextarea = await screen.findByLabelText(/pgn move input/i);
    await waitFor(() => {
      expect(uploadTextarea).toHaveValue('1. e4 e5 2.d4');
    });

    await user.type(uploadTextarea, ' e5');
    await waitFor(() => {
      expect(uploadTextarea).toHaveValue('1. e4 e5 2.d4 e5');
    });

    const uploadAlert = await screen.findByRole('alert');
    expect(uploadAlert.textContent).toBe(pasteMessage);
  });

  it('falls back to FileReader when File.text is unavailable', async () => {
    const user = userEvent.setup();
    const onImportLine = vi.fn(
      (): ImportResult => ({
        added: true,
        line: {
          id: 'fallback-line',
          opening: 'Danish Gambit',
          color: 'White',
          moves: ['e4', 'e5', 'd4', 'exd4', 'c3'],
          display: '1.e4 e5 2.d4 exd4 3.c3',
          scheduledFor: new Date().toISOString(),
        },
      }),
    );

    const readAsText = vi.fn(function (
      this: { onload: ((event: ProgressEvent<FileReader>) => void) | null; result: string },
      file: Blob,
    ) {
      void file;
      this.result = '1.e4 e5 2.d4 exd4 3.c3';
      this.onload?.({
        target: this as unknown as FileReader,
      } as ProgressEvent<FileReader>);
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

    render(<PgnImportPane onImportLine={onImportLine} />);

    const handle = screen.getByRole('button', { name: /open pgn import tools/i });
    fireEvent.pointerEnter(handle);

    const uploadOption = await screen.findByRole('button', { name: /upload pgn/i });
    await user.click(uploadOption);

    const fileInput = await screen.findByLabelText(/upload pgn file/i);
    const fallbackFile = {
      name: 'fallback.pgn',
      type: 'application/x-chess-pgn',
      size: 29,
    } as File;

    Object.defineProperty(fileInput, 'files', {
      configurable: true,
      value: [fallbackFile],
    });
    fireEvent.change(fileInput);

    const confirmButton = await screen.findByRole('button', {
      name: /add to danish gambit \(white\)/i,
    });
    await user.click(confirmButton);

    expect(onImportLine).toHaveBeenCalledWith({
      opening: 'Danish Gambit',
      color: 'White',
      moves: ['e4', 'e5', 'd4', 'exd4', 'c3'],
      display: '1.e4 e5 2.d4 exd4 3.c3',
    });
    expect(readAsText).toHaveBeenCalledWith(fallbackFile);
  });

  it('notifies the user when an uploaded PGN cannot be read', async () => {
    const user = userEvent.setup();

    render(
      <PgnImportPane
        onImportLine={() => ({
          added: false,
          line: {
            id: 'unreadable-file',
            opening: 'Danish Gambit',
            color: 'White',
            moves: [],
            display: '',
            scheduledFor: new Date().toISOString(),
          },
        })}
      />,
    );

    const handle = screen.getByRole('button', { name: /open pgn import tools/i });
    fireEvent.pointerEnter(handle);

    const uploadOption = await screen.findByRole('button', { name: /upload pgn/i });
    await user.click(uploadOption);

    const fileInput = await screen.findByLabelText(/upload pgn file/i);
    const brokenFile = new File(['1. e4 e5 2.d4'], 'broken.pgn', {
      type: 'application/x-chess-pgn',
    });
    Object.defineProperty(brokenFile, 'text', {
      configurable: true,
      value: () => Promise.reject(new Error('cannot read file')),
    });

    await user.upload(fileInput, brokenFile);

    const feedback = await screen.findByRole('alert');
    expect(feedback).toHaveTextContent('We could not read that PGN file. Please try again.');
  });

  it('ignores upload change events when no file is selected', async () => {
    const user = userEvent.setup();
    const onImportLine = vi.fn(
      (): ImportResult => ({
        added: true,
        line: {
          id: 'unused',
          opening: 'Danish Gambit',
          color: 'White',
          moves: [],
          display: '',
          scheduledFor: new Date().toISOString(),
        },
      }),
    );

    render(<PgnImportPane onImportLine={onImportLine} />);

    const handle = screen.getByRole('button', { name: /open pgn import tools/i });
    fireEvent.pointerEnter(handle);

    const uploadOption = await screen.findByRole('button', { name: /upload pgn/i });
    await user.click(uploadOption);

    const fileInput = await screen.findByLabelText(/upload pgn file/i);
    Object.defineProperty(fileInput, 'files', {
      configurable: true,
      value: null,
    });

    fireEvent.change(fileInput);

    expect(onImportLine).not.toHaveBeenCalled();
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });
});
