import type { JSX } from 'react';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';

import './App.css';
import { CommandConsole } from './components/CommandConsole';
import { SessionRoutes } from './components/SessionRoutes';
import type { DetectedOpeningLine, ImportResult, ScheduledOpeningLine } from './types/repertoire';
import { createCommandDispatcher } from './utils/commandDispatcher';
import type { CommandDispatcher, CommandHandler } from './utils/commandDispatcher';
import { createOpeningLineScheduler, linesMatch } from './utils/importedLines';

const isTextEntryTarget = (target: EventTarget | null): boolean => {
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  if (target.isContentEditable) {
    return true;
  }

  const tagName = target.tagName;
  return tagName === 'INPUT' || tagName === 'TEXTAREA' || target.getAttribute('role') === 'textbox';
};

const isColonKeyPressed = (event: KeyboardEvent): boolean => {
  // Detects colon key (:) on US keyboards: Shift+; or direct colon key
  return event.key === ':' || (event.key === ';' && event.shiftKey);
};

const App = (): JSX.Element => {
  const [isConsoleOpen, setIsConsoleOpen] = useState(false);
  const [importedLines, setImportedLines] = useState<ScheduledOpeningLine[]>([]);
  const navigate = useNavigate();

  const handleOpenConsole = useCallback(() => {
    setIsConsoleOpen(true);
  }, []);

  const handleCloseConsole = useCallback(() => {
    setIsConsoleOpen(false);
  }, []);

  const dispatcher: CommandDispatcher = useMemo(() => {
    const buildNavigationHandler = (path: string): CommandHandler => {
      return () => {
        void navigate(path);
        return undefined;
      };
    };

    return createCommandDispatcher({
      onUnknownCommand: (input) => {
        window.alert(input);
      },
      commands: [
        { command: 'cb', handler: buildNavigationHandler('/tools/board') },
        { command: 's', handler: buildNavigationHandler('/review/opening') },
        { command: 'db', handler: buildNavigationHandler('/dashboard') },
      ],
    });
  }, [navigate]);

  const scheduleOpeningLine = useMemo(() => createOpeningLineScheduler(), []);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const isColonKey = isColonKeyPressed(event);

      if (isColonKey && !event.metaKey && !event.ctrlKey && !event.altKey) {
        if (isTextEntryTarget(event.target)) {
          return;
        }

        event.preventDefault();
        handleOpenConsole();
        return;
      }

      if (event.key === 'Escape' && isConsoleOpen) {
        event.preventDefault();
        handleCloseConsole();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [handleCloseConsole, handleOpenConsole, isConsoleOpen]);

  const handleImportLine = (line: DetectedOpeningLine): ImportResult => {
    const existing = importedLines.find((candidate) => linesMatch(candidate, line));
    if (existing) {
      return { added: false, line: existing };
    }

    const nextLine = scheduleOpeningLine(line, importedLines.length);
    setImportedLines((previous) => [...previous, nextLine]);
    return { added: true, line: nextLine };
  };

  const handleExecuteCommand = useCallback(
    async (input: string) => {
      await dispatcher.dispatch(input);
    },
    [dispatcher],
  );

  return (
    <>
      <SessionRoutes
        importedLines={importedLines}
        onImportLine={handleImportLine}
        commandDispatcher={dispatcher}
      />
      <CommandConsole
        isOpen={isConsoleOpen}
        onOpen={handleOpenConsole}
        onClose={handleCloseConsole}
        onExecuteCommand={handleExecuteCommand}
      />
    </>
  );
};

export default App;
