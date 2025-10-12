export type CommandContext = {
  source: 'palette' | 'shortcut' | 'api';
  issuedAt: Date;
};

export type CommandRegistration = {
  id: string;
  title: string;
  keywords?: string[];
  category?: string;
  description?: string;
};

export type CommandExecution = {
  command: CommandRegistration;
  context: CommandContext;
  status: 'success' | 'error';
  message?: string;
};

export type CommandHandler = (
  context: CommandContext,
) => Promise<CommandExecution | undefined> | CommandExecution | undefined;

export type CommandPaletteService = {
  register: (command: CommandRegistration, handler: CommandHandler) => void;
  unregister: (commandId: string) => void;
  list: () => CommandRegistration[];
  execute: (
    commandId: string,
    context?: Partial<CommandContext>,
  ) => Promise<CommandExecution>;
  subscribe: (listener: (execution: CommandExecution) => void) => () => void;
  reset: () => void;
};
