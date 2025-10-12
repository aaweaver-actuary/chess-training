import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import type {
  CommandHandler,
  CommandPaletteService,
  CommandRegistration,
} from '../services/CommandPaletteService.js';
import type {
  PgnImportFeedbackMessage,
  PgnImportOutcome,
  PgnImportPreview,
  PgnImportService,
  PgnImportSource,
} from '../services/PgnImportService.js';

export type ImportMode = 'idle' | 'paste' | 'upload';

export type UsePgnImportOptions = {
  service: PgnImportService;
  commandPalette?: CommandPaletteService;
};

export type UsePgnImportState = {
  isExpanded: boolean;
  mode: ImportMode;
  sourceText: string;
  preview?: PgnImportPreview;
  messages: PgnImportFeedbackMessage[];
  errors: string[];
};

export type UsePgnImportActions = {
  activate: (mode: Exclude<ImportMode, 'idle'>) => void;
  collapse: () => void;
  setSourceText: (value: string) => Promise<void>;
  importFromFile: (file: Blob | File) => Promise<void>;
  acknowledgeMessages: () => void;
};

const collapseRegistration: CommandRegistration = {
  id: 'pgn-import:collapse',
  title: 'Close PGN import tools',
  keywords: ['pgn', 'import', 'close'],
  category: 'Import',
  description: 'Collapse the PGN import pane.',
};

const isEmpty = (value: string): boolean => value.trim().length === 0;

export const usePgnImport = ({
  service,
  commandPalette,
}: UsePgnImportOptions): { state: UsePgnImportState; actions: UsePgnImportActions } => {
  const [isExpanded, setIsExpanded] = useState(false);
  const [mode, setMode] = useState<ImportMode>('idle');
  const [sourceText, setSourceTextState] = useState('');
  const [preview, setPreview] = useState<PgnImportPreview | undefined>(undefined);
  const [messages, setMessages] = useState<PgnImportFeedbackMessage[]>([]);
  const [errors, setErrors] = useState<string[]>([]);
  const lastOutcomeRef = useRef<PgnImportOutcome | undefined>(undefined);

  const resetFeedback = useCallback(() => {
    setPreview(undefined);
    setMessages([]);
    setErrors([]);
    lastOutcomeRef.current = undefined;
  }, []);

  const collapse = useCallback(() => {
    setIsExpanded(false);
    setMode('idle');
    setSourceTextState('');
    resetFeedback();
    service.clear();
  }, [resetFeedback, service]);

  const applyOutcome = useCallback((outcome: PgnImportOutcome) => {
    lastOutcomeRef.current = outcome;
    setPreview(outcome.preview);
    setMessages(outcome.messages);
    setErrors(outcome.errors);
  }, []);

  const detectSource = useCallback(
    async (source: PgnImportSource): Promise<void> => {
      const outcome = await service.detect(source);
      applyOutcome(outcome);
    },
    [applyOutcome, service],
  );

  const activate = useCallback(
    (nextMode: Exclude<ImportMode, 'idle'>) => {
      setMode(nextMode);
      setIsExpanded(true);
      setSourceTextState('');
      resetFeedback();
    },
    [resetFeedback],
  );

  const setSourceTextAction = useCallback(
    async (value: string) => {
      setIsExpanded(true);
      setSourceTextState(value);

      if (isEmpty(value)) {
        resetFeedback();
        return;
      }

      await detectSource({ kind: 'text', value });
    },
    [detectSource, resetFeedback],
  );

  const importFromFile = useCallback(
    async (file: Blob | File) => {
      setIsExpanded(true);
      setMode('upload');

      await detectSource({ kind: 'file', value: file });

      const outcome = lastOutcomeRef.current;
      if (outcome) {
        setSourceTextState(outcome.preview.normalizedPgn);
      }
    },
    [detectSource],
  );

  const acknowledgeMessages = useCallback(() => {
    const outcome = lastOutcomeRef.current;
    if (!outcome || outcome.messages.length === 0) {
      setMessages([]);
      return;
    }

    service.acknowledge(outcome);
    lastOutcomeRef.current = {
      ...outcome,
      messages: [],
    } satisfies PgnImportOutcome;
    setMessages([]);
  }, [service]);

  useEffect(() => {
    if (!commandPalette) {
      return undefined;
    }

    const handler: CommandHandler = async () => {
      collapse();
      return undefined;
    };

    commandPalette.register(collapseRegistration, handler);
    return () => {
      commandPalette.unregister(collapseRegistration.id);
    };
  }, [commandPalette, collapse]);

  const state = useMemo<UsePgnImportState>(
    () => ({
      isExpanded,
      mode,
      sourceText,
      preview,
      messages,
      errors,
    }),
    [errors, isExpanded, messages, mode, preview, sourceText],
  );

  const actions = useMemo<UsePgnImportActions>(
    () => ({
      activate,
      collapse,
      setSourceText: setSourceTextAction,
      importFromFile,
      acknowledgeMessages,
    }),
    [acknowledgeMessages, activate, collapse, importFromFile, setSourceTextAction],
  );

  return { state, actions };
};
