import { describe, expect, it, vi } from 'vitest';

import { createCommandDispatcher } from '../commandDispatcher';

describe('createCommandDispatcher', () => {
  it('registers initial commands from options', async () => {
    const handler = vi.fn();
    const onUnknownCommand = vi.fn();
    const dispatcher = createCommandDispatcher({
      onUnknownCommand,
      commands: [{ command: 'greet', handler }],
    });

    await dispatcher.dispatch('greet world');

    expect(handler).toHaveBeenCalledWith('greet', ['world']);
    expect(onUnknownCommand).not.toHaveBeenCalled();
  });

  it('handles duplicate command registrations in initial commands', async () => {
    const firstHandler = vi.fn();
    const secondHandler = vi.fn();
    const onUnknownCommand = vi.fn();
    const dispatcher = createCommandDispatcher({
      onUnknownCommand,
      commands: [
        { command: 'echo', handler: firstHandler },
        { command: 'echo', handler: secondHandler },
      ],
    });

    await dispatcher.dispatch('echo test');

    expect(secondHandler).toHaveBeenCalledWith('echo', ['test']);
    expect(firstHandler).not.toHaveBeenCalled();
    expect(onUnknownCommand).not.toHaveBeenCalled();
  });
});
