import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { CommandConsole } from '../CommandConsole';

describe('CommandConsole', () => {
  const renderConsole = (overrides: Partial<React.ComponentProps<typeof CommandConsole>> = {}) => {
    const onClose = vi.fn();
    const onExecuteCommand = vi.fn<(command: string) => Promise<void>>();
    onExecuteCommand.mockResolvedValue(undefined);
    const onOpen = vi.fn();

    render(
      <CommandConsole
        isOpen
        onClose={onClose}
        onExecuteCommand={onExecuteCommand}
        onOpen={onOpen}
        {...overrides}
      />,
    );

    return { onClose, onExecuteCommand, onOpen };
  };

  it('closes when submitting an empty command without modifiers', () => {
    const { onClose, onExecuteCommand } = renderConsole();

    const input = screen.getByLabelText<HTMLInputElement>(/command input/i);
    const form = input.closest('form');
    expect(form).not.toBeNull();
    if (!form) {
      throw new Error('Expected command console form');
    }

    fireEvent.change(input, { target: { value: '   ' } });
    fireEvent.submit(form);

    expect(onExecuteCommand).not.toHaveBeenCalled();
    expect(onClose).toHaveBeenCalledTimes(1);
    expect(input.value).toBe('');
  });

  it('keeps focus when submitting an empty command with Ctrl+Enter', () => {
    const { onClose } = renderConsole();

    const input = screen.getByLabelText<HTMLInputElement>(/command input/i);
    const form = input.closest('form');
    expect(form).not.toBeNull();
    if (!form) {
      throw new Error('Expected command console form');
    }

    fireEvent.change(input, { target: { value: '   ' } });
    fireEvent.keyDown(input, { key: 'Enter', ctrlKey: true });
    fireEvent.submit(form);

    expect(onClose).not.toHaveBeenCalled();
    expect(document.activeElement).toBe(input);
  });

  it('clears the command and closes after executing without modifiers', async () => {
    const { onClose, onExecuteCommand } = renderConsole();
    const user = userEvent.setup();

    const input = screen.getByLabelText<HTMLInputElement>(/command input/i);
    await user.type(input, 'help');

    await user.keyboard('{Enter}');

    await waitFor(() => {
      expect(onExecuteCommand).toHaveBeenCalledWith('help');
    });
    expect(onClose).toHaveBeenCalledTimes(1);
    expect(input.value).toBe('');
  });

  it('keeps the console open when executing with Ctrl+Enter', async () => {
    const { onClose, onExecuteCommand } = renderConsole();
    const user = userEvent.setup();

    const input = screen.getByLabelText<HTMLInputElement>(/command input/i);
    await user.type(input, 'status');

    fireEvent.keyDown(input, { key: 'Enter', metaKey: true });
    const form = input.closest('form');
    expect(form).not.toBeNull();
    if (!form) {
      throw new Error('Expected command console form');
    }
    fireEvent.submit(form);

    await waitFor(() => {
      expect(onExecuteCommand).toHaveBeenCalledWith('status');
    });
    expect(onClose).not.toHaveBeenCalled();
    expect(input.value).toBe('');
    expect(document.activeElement).toBe(input);
  });
});
