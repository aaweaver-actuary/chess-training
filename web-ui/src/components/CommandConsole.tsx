import { useEffect, useMemo, useRef, useState } from 'react';
import type { FormEvent, KeyboardEvent } from 'react';

import './CommandConsole.css';

type CommandConsoleProps = {
  isOpen: boolean;
  onOpen: () => void;
  onClose: () => void;
  onExecuteCommand: (command: string) => Promise<void> | void;
};

const CONSOLE_ANIMATION_DURATION_MS = 240;

export const CommandConsole = ({
  isOpen,
  onOpen,
  onClose,
  onExecuteCommand,
}: CommandConsoleProps) => {
  const [isRendered, setIsRendered] = useState(isOpen);
  const [isClosing, setIsClosing] = useState(false);
  const [command, setCommand] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);
  const keepOpenRef = useRef(false);

  useEffect(() => {
    if (isOpen) {
      setIsRendered(true);
      setIsClosing(false);
      return;
    }

    if (!isRendered) {
      return;
    }

    setIsClosing(true);
    const timeout = window.setTimeout(() => {
      setIsRendered(false);
      setIsClosing(false);
    }, CONSOLE_ANIMATION_DURATION_MS);

    return () => {
      window.clearTimeout(timeout);
    };
  }, [isOpen, isRendered]);

  const launcherClassName = useMemo(() => {
    const classNames = ['command-console-launcher'];
    if (isRendered) {
      classNames.push('command-console-launcher--active');
    }
    return classNames.join(' ');
  }, [isRendered]);

  useEffect(() => {
    if (!isOpen) {
      setCommand('');
    }
  }, [isOpen]);

  useEffect(() => {
    if (!isOpen) {
      return;
    }

    if (inputRef.current) {
      inputRef.current.focus();
    }
  }, [isOpen]);

  const overlayClassName = useMemo(() => {
    const classNames = ['command-console-overlay'];
    if (isClosing) {
      classNames.push('command-console-overlay--closing');
    }
    return classNames.join(' ');
  }, [isClosing]);

  const consoleClassName = useMemo(() => {
    const classNames = ['command-console'];
    if (isClosing) {
      classNames.push('command-console--closing');
    }
    return classNames.join(' ');
  }, [isClosing]);

  const isVisible = isOpen || isClosing;
  const shouldRenderConsole = isRendered || isOpen;

  const resetFocus = () => {
    if (inputRef.current) {
      inputRef.current.focus();
    }
  };

  const handleKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter') {
      keepOpenRef.current = event.ctrlKey || event.metaKey;
    }
  };

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const keepOpen = keepOpenRef.current;
    keepOpenRef.current = false;

    const trimmed = command.trim();
    setCommand('');

    if (trimmed.length === 0) {
      if (keepOpen) {
        resetFocus();
        return;
      }
      onClose();
      return;
    }

    await onExecuteCommand(trimmed);

    if (keepOpen) {
      resetFocus();
      return;
    }

    onClose();
  };

  return (
    <>
      <button
        type="button"
        className={launcherClassName}
        onClick={isVisible ? onClose : onOpen}
        aria-label={isVisible ? 'Close command console' : 'Open command console'}
        aria-expanded={isVisible}
      >
        <span className="command-console-launcher__icon">{isVisible ? '×' : '$:'}</span>
      </button>
      {shouldRenderConsole && (
        <div className={overlayClassName}>
          <div
            className={consoleClassName}
            role="dialog"
            aria-label="Command console"
            aria-modal="true"
          >
            <div className="command-console__container">
              <div className="command-console__header">
                <div className="command-console__header-content">
                  <span className="command-console__title">Command Console</span>
                  <span className="command-console__hint">Press Esc or click × to close</span>
                </div>
                <button
                  type="button"
                  className="command-console__close"
                  onClick={onClose}
                  aria-label="Close command console"
                >
                  ×
                </button>
              </div>
              <form
                className="command-console__body"
                onSubmit={(event) => {
                  void handleSubmit(event);
                }}
              >
                <span className="command-console__prompt">$:</span>
                <input
                  ref={inputRef}
                  type="text"
                  className="command-console__input"
                  aria-label="Command input"
                  placeholder="Awaiting commands…"
                  value={command}
                  onChange={(event) => {
                    setCommand(event.target.value);
                  }}
                  onKeyDown={handleKeyDown}
                />
              </form>
            </div>
          </div>
        </div>
      )}
    </>
  );
};
