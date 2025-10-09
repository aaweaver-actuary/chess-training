export type CommandHandlerResult = void | string | Promise<void | string>;

export type CommandHandler = (command: string, args: string[]) => CommandHandlerResult;

export type CommandDispatcher = {
  register: (command: string, handler: CommandHandler) => void;
  dispatch: (input: string) => Promise<void>;
};

export type CommandDispatcherOptions = {
  onUnknownCommand: (command: string) => void;
  onResult?: (message: string) => void;
};

export const createCommandDispatcher = ({
  onUnknownCommand,
  onResult,
}: CommandDispatcherOptions): CommandDispatcher => {
  const handlers = new Map<string, CommandHandler>();

  const register = (command: string, handler: CommandHandler) => {
    const key = command.trim().toLowerCase();
    if (!key) {
      return;
    }
    handlers.set(key, handler);
  };

  const dispatch = async (input: string) => {
    const trimmed = input.trim();
    if (!trimmed) {
      return;
    }

    const [command, ...args] = trimmed.split(/\s+/);
    const handler = handlers.get(command.toLowerCase());

    if (!handler) {
      onUnknownCommand(trimmed);
      return;
    }

    const result = await handler(command, args);

    if (typeof result === 'string' && result.trim().length > 0) {
      onResult?.(result);
    }
  };

  return { register, dispatch };
};
