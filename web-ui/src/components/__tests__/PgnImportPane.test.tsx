import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { CommandDispatcher } from '../../utils/commandDispatcher';
import type { ImportResult } from '../../types/repertoire';
import { PgnImportPane } from '../PgnImportPane';

describe('PgnImportPane', () => {
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

  it('recognizes a King\'s Knight Opening line from a short PGN import', async () => {
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
});
