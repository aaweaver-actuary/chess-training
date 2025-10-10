import type { FocusEvent, JSX } from 'react';
import { useCallback, useEffect, useRef, useState } from 'react';

import './PgnImportPane.css';
import type { DetectedOpeningLine, ImportResult } from '../types/repertoire';
import { formatUnlockDate, UNLOCK_DATE_FALLBACK_LABEL } from '../utils/formatUnlockDate';
import type { CommandDispatcher, CommandHandler } from '../utils/commandDispatcher';

type PgnImportPaneProps = {
  onImportLine: (line: DetectedOpeningLine) => ImportResult;
  commandDispatcher?: CommandDispatcher;
};

type FeedbackState =
  | { kind: 'success' | 'info'; message: string }
  | { kind: 'error'; message: string }
  | undefined;

const DANISH_PATTERN = ['e4', 'e5', 'd4', 'exd4', 'c3'] as const;

const sanitizeMoves = (input: string): string[] =>
  input
    .replace(/\s+/g, ' ')
    .split(' ')
    .map((token) => token.trim())
    .map((token) => token.replace(/^[0-9]+\.\.\./, ''))
    .map((token) => token.replace(/^[0-9]+\./, ''))
    .map((token) => token.replace(/[?!+#]/g, ''))
    .filter((token) => token.length > 0);

const formatMoveSequence = (moves: string[]): string => {
  const segments: string[] = [];
  for (let index = 0; index < moves.length; index += 2) {
    const moveNumber = Math.floor(index / 2) + 1;
    const whiteMove = moves[index];
    const blackMove = moves[index + 1];
    if (whiteMove) {
      segments.push(`${String(moveNumber)}.${whiteMove}`);
    }
    if (blackMove) {
      segments.push(blackMove);
    }
  }
  return segments.join(' ');
};

const detectOpening = (input: string): DetectedOpeningLine | undefined => {
  const moves = sanitizeMoves(input);
  if (moves.length < DANISH_PATTERN.length) {
    return undefined;
  }

  const normalized = moves.map((move) => move.toLowerCase());
  const isDanish = DANISH_PATTERN.every(
    (expectedMove, index) => normalized[index] === expectedMove,
  );

  if (!isDanish) {
    return undefined;
  }

  return {
    opening: 'Danish Gambit',
    color: 'White',
    moves,
    display: formatMoveSequence(moves),
  } satisfies DetectedOpeningLine;
};

const buildScheduledMessage = (result: ImportResult): FeedbackState => {
  const { line, added } = result;
  const friendlyDate = formatUnlockDate(line.scheduledFor);
  const lowerColor = line.color.toLowerCase();

  if (friendlyDate === UNLOCK_DATE_FALLBACK_LABEL) {
    return {
      kind: added ? 'success' : 'info',
      message: added
        ? `Line added to your ${lowerColor} ${line.opening} repertoire.`
        : `This ${line.opening} line is already part of your ${lowerColor} repertoire.`,
    };
  }

  return {
    kind: added ? 'success' : 'info',
    message: added
      ? `Scheduled for ${friendlyDate} in your ${lowerColor} ${line.opening} repertoire.`
      : `Already scheduled for ${friendlyDate} in your ${lowerColor} ${line.opening} repertoire.`,
  };
};

export const PgnImportPane = ({
  onImportLine,
  commandDispatcher,
}: PgnImportPaneProps): JSX.Element => {
  const containerRef = useRef<HTMLElement | null>(null);
  const [isExpanded, setIsExpanded] = useState(false);
  const [isPasteMode, setIsPasteMode] = useState(false);
  const [pgnText, setPgnText] = useState('');
  const [detectedLine, setDetectedLine] = useState<DetectedOpeningLine | undefined>(undefined);
  const [feedback, setFeedback] = useState<FeedbackState>(undefined);

  const collapsePane = useCallback(() => {
    setIsExpanded(false);
    setIsPasteMode(false);
  }, []);

  const paneContainsFocus = () => {
    const active = document.activeElement;
    return Boolean(active && containerRef.current?.contains(active));
  };

  const handlePointerEnter = () => {
    setIsExpanded(true);
  };

  const handleFocusCapture = () => {
    setIsExpanded(true);
  };

  const handleBlurCapture = (event: FocusEvent<HTMLElement>) => {
    const nextTarget = event.relatedTarget as Node | null;
    if (nextTarget && containerRef.current?.contains(nextTarget)) {
      return;
    }

    if (!paneContainsFocus()) {
      collapsePane();
    }
  };

  const handlePasteOption = () => {
    setIsPasteMode(true);
    setIsExpanded(true);
  };

  const handlePgnChange = (value: string) => {
    setPgnText(value);
    setFeedback(undefined);

    if (value.trim().length === 0) {
      setDetectedLine(undefined);
      setFeedback(undefined);
      return;
    }

    const line = detectOpening(value);
    if (line) {
      setDetectedLine(line);
      setFeedback(undefined);
      return;
    }

    setDetectedLine(undefined);
    setFeedback({
      kind: 'error',
      message: 'We could not recognize that PGN yet. Try a standard Danish Gambit line.',
    });
  };

  const handleConfirm = (line: DetectedOpeningLine) => {
    const result = onImportLine(line);
    const message = buildScheduledMessage(result);
    setFeedback(message);

    if (result.added) {
      setPgnText('');
      setDetectedLine(undefined);
    }
  };

  useEffect(() => {
    if (!isExpanded) {
      return;
    }

    const handlePointerDown = (event: PointerEvent) => {
      const container = containerRef.current;
      /* c8 ignore next 3 -- React assigns the container ref before this handler runs */
      if (!container) {
        return;
      }

      const path = event.composedPath();
      if (path.includes(container)) {
        return;
      }

      collapsePane();
    };

    window.addEventListener('pointerdown', handlePointerDown);
    return () => {
      window.removeEventListener('pointerdown', handlePointerDown);
    };
  }, [collapsePane, isExpanded]);

  useEffect(() => {
    if (!commandDispatcher) {
      return;
    }

    const handler: CommandHandler = () => {
      collapsePane();
      return undefined;
    };

    commandDispatcher.register('x', handler);
    return () => {
      commandDispatcher.unregister?.('x');
    };
  }, [collapsePane, commandDispatcher]);

  return (
    <aside
      ref={containerRef}
      className={`pgn-import-pane${isExpanded ? ' pgn-import-pane-expanded' : ''}`}
      aria-label="PGN import tools"
      onPointerEnter={handlePointerEnter}
      onFocusCapture={handleFocusCapture}
      onBlurCapture={handleBlurCapture}
    >
      <button
        type="button"
        className="pgn-import-handle"
        aria-label="Open PGN import tools"
        aria-expanded={isExpanded}
        onPointerEnter={handlePointerEnter}
        onFocus={handlePointerEnter}
      >
        Import PGN
      </button>
      <div className="pgn-import-content" aria-hidden={!isExpanded} hidden={!isExpanded}>
        <header className="pgn-import-header">
          <h2>Import lines</h2>
          <p className="pgn-import-subtitle">Grow your repertoire from existing PGNs.</p>
        </header>
        <div className="pgn-import-body">
          <button
            type="button"
            className="pgn-import-option"
            onClick={handlePasteOption}
            aria-pressed={isPasteMode}
          >
            Paste PGN
          </button>

          {isPasteMode ? (
            <div className="pgn-import-form" role="region" aria-label="Paste PGN">
              <label className="pgn-import-label" htmlFor="pgn-import-textarea">
                Paste moves
              </label>
              <textarea
                id="pgn-import-textarea"
                value={pgnText}
                onChange={(event) => {
                  handlePgnChange(event.target.value);
                }}
                placeholder="1.e4 e5 2.d4 exd4 3.c3"
                aria-label="PGN move input"
              />
              {detectedLine ? (
                <div className="pgn-import-detection" role="status">
                  <p>
                    Detected <strong>{detectedLine.opening}</strong> for the{' '}
                    <strong>{detectedLine.color.toLowerCase()}</strong> pieces.
                  </p>
                  <p className="pgn-import-preview">{detectedLine.display}</p>
                  <button
                    type="button"
                    className="pgn-import-confirm"
                    onClick={() => {
                      handleConfirm(detectedLine);
                    }}
                  >
                    Add to {detectedLine.opening} ({detectedLine.color})
                  </button>
                </div>
              ) : null}
            </div>
          ) : null}
        </div>
        {feedback ? (
          <p
            className={`pgn-import-feedback pgn-import-feedback-${feedback.kind}`}
            role={feedback.kind === 'error' ? 'alert' : 'status'}
          >
            {feedback.message}
          </p>
        ) : null}
      </div>
    </aside>
  );
};
