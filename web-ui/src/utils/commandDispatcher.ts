export type CommandHandlerResult = string | undefined | Promise<string | undefined>;

export type CommandHandler = (command: string, args: string[]) => CommandHandlerResult;

export type CommandDispatcher = {
  register: (command: string, handler: CommandHandler) => boolean;
  unregister?: (command: string) => void;
  dispatch: (input: string) => Promise<void>;
};

export type CommandDispatcherOptions = {
  onUnknownCommand: (command: string) => void;
  onResult?: (message: string) => void;
  commands?: Array<{ command: string; handler: CommandHandler }>;
};

export const createCommandDispatcher = ({
  onUnknownCommand,
  onResult,
  commands,
}: CommandDispatcherOptions): CommandDispatcher => {
  const handlers = new Map<string, CommandHandler>();

  const register = (command: string, handler: CommandHandler): boolean => {
    const key = command.trim().toLowerCase();
    if (!key) {
      return false;
    }
    handlers.set(key, handler);
    return true;
  };

  commands?.forEach(({ command, handler }) => {
    register(command, handler);
  });

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
