import { vi, describe, it, expect } from 'vitest';
import { createCommandDispatcher } from '../commandDispatcher';

const createDispatcher = () => {
  const onUnknownCommand = vi.fn();
  const onResult = vi.fn();

  return {
    onUnknownCommand,
    onResult,
    dispatcher: createCommandDispatcher({ onUnknownCommand, onResult }),
  };
};

describe('createCommandDispatcher', () => {
  it('rejects blank command registrations', async () => {
    const { dispatcher, onUnknownCommand } = createDispatcher();

    expect(dispatcher.register('', vi.fn())).toBe(false);
    expect(dispatcher.register('   ', vi.fn())).toBe(false);

    await dispatcher.dispatch('');
    expect(onUnknownCommand).not.toHaveBeenCalled();
  });

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
  it('dispatches matching handlers case-insensitively', async () => {
    const { dispatcher } = createDispatcher();
    const handler = vi.fn();

    expect(dispatcher.register('Ping', handler)).toBe(true);

    await dispatcher.dispatch('  ping   PONG  ');

    expect(handler).toHaveBeenCalledWith('ping', ['PONG']);
  });

  it('notifies when a command is unknown', async () => {
    const { dispatcher, onUnknownCommand } = createDispatcher();

    await dispatcher.dispatch('missing command');

    expect(onUnknownCommand).toHaveBeenCalledWith('missing command');
  });

  it('notifies when an unknown command has extra whitespace', async () => {
    const { dispatcher, onUnknownCommand } = createDispatcher();

    await dispatcher.dispatch('   missing command   ');

    expect(onUnknownCommand).toHaveBeenCalledWith('missing command');
  });

  it('emits result messages only when handler returns a non-empty string', async () => {
    const { dispatcher, onResult } = createDispatcher();

    dispatcher.register('echo', () => '   hello world  ');
    dispatcher.register('silent', () => undefined);
    dispatcher.register('blank', () => '   ');

    await dispatcher.dispatch('echo');
    await dispatcher.dispatch('silent');
    await dispatcher.dispatch('blank');

    expect(onResult).toHaveBeenCalledTimes(1);
    expect(onResult).toHaveBeenCalledWith('   hello world  ');
  });

  it('awaits asynchronous handler results', async () => {
    const { dispatcher, onResult } = createDispatcher();

    dispatcher.register('async', async () => {
      await new Promise((resolve) => setTimeout(resolve, 0));
      return 'done';
    });

    await dispatcher.dispatch('async');

    expect(onResult).toHaveBeenCalledWith('done');
  });
});
