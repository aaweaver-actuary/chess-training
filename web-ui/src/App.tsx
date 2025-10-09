import type { JSX } from 'react';

import { useCallback, useEffect, useMemo, useRef, useState, useSyncExternalStore } from 'react';
import { Navigate, Route, Routes, useNavigate } from 'react-router-dom';

import './App.css';
import { sampleSnapshot } from './fixtures/sampleSnapshot';
import { DashboardPage } from './pages/DashboardPage';
import { BlankBoardPage } from './pages/BlankBoardPage';
import { OpeningReviewPage } from './pages/OpeningReviewPage';
import { ReviewPlanner } from './services/ReviewPlanner';
import type { ReviewOverview } from './services/ReviewPlanner';
import type { CardSummary, ReviewGrade } from './types/gateway';
import { sessionStore } from './state/sessionStore';
import { CommandConsole } from './components/CommandConsole';
import type { DetectedOpeningLine, ImportResult, ScheduledOpeningLine } from './types/repertoire';
import { createCommandDispatcher } from './utils/commandDispatcher';
import type { CommandDispatcher } from './utils/commandDispatcher';

const planner = new ReviewPlanner();
const baselineOverview = planner.buildOverview(sampleSnapshot);

const useSessionState = () =>
  useSyncExternalStore(sessionStore.subscribe, sessionStore.getState, sessionStore.getState);

const buildOverview = (stats: ReturnType<typeof sessionStore.getState>['stats']) => {
  const baseline = baselineOverview;
  if (!stats) {
    return baseline;
  }

  const totalDue = stats.due_count;
  const completed = stats.completed_count;
  const remaining = Math.max(totalDue - completed, 0);
  const completionRate = totalDue === 0 ? 1 : completed / totalDue;

  return {
    ...baseline,
    progress: {
      ...baseline.progress,
      totalDue,
      completedToday: completed,
      remaining,
      completionRate,
      accuracyRate: stats.accuracy,
    },
  };
};

const scheduleDateForLine = (offset: number): string => {
  const base = new Date();
  base.setDate(base.getDate() + 1 + offset);
  return base.toISOString().slice(0, 10);
};

const scheduleOpeningLine = (line: DetectedOpeningLine, offset: number): ScheduledOpeningLine => ({
  ...line,
  id: ['import', Date.now().toString(), offset.toString()].join('-'),
  scheduledFor: scheduleDateForLine(offset),
});

const linesMatch = (candidate: ScheduledOpeningLine, target: DetectedOpeningLine): boolean =>
  candidate.opening === target.opening &&
  candidate.color === target.color &&
  candidate.moves.length === target.moves.length &&
  candidate.moves.every((move, index) => move.toLowerCase() === target.moves[index]?.toLowerCase());

const augmentOverviewWithImports = (
  overview: ReviewOverview,
  lines: ScheduledOpeningLine[],
): ReviewOverview => ({
  ...overview,
  upcomingUnlocks: [
    ...overview.upcomingUnlocks,
    ...lines.map((line) => ({
      id: line.id,
      move: `${line.opening} (${line.color})`,
      idea: `Line: ${line.display}`,
      scheduledFor: line.scheduledFor,
    })),
  ],
});

type SessionRoutesProps = {
  importedLines: ScheduledOpeningLine[];
  onImportLine: (line: DetectedOpeningLine) => ImportResult;
  commandDispatcher?: CommandDispatcher;
};

const useStartTimestamp = (card?: CardSummary) => {
  const startedAtRef = useRef<number>(performance.now());
  useEffect(() => {
    if (card) {
      startedAtRef.current = performance.now();
    }
  }, [card]);
  return startedAtRef;
};

const useSessionLifecycle = (start: (userId: string) => Promise<void>) => {
  useEffect(() => {
    void start('demo-user');
  }, [start]);
};

const SessionRoutes = ({ importedLines, onImportLine, commandDispatcher }: SessionRoutesProps) => {
  const session = useSessionState();
  const { stats, currentCard, start, submitGrade } = session;
  useSessionLifecycle(start);

  const startedAtRef = useStartTimestamp(currentCard);

  const overview = useMemo(() => buildOverview(stats), [stats]);
  const dashboardOverview = useMemo(
    () => augmentOverviewWithImports(overview, importedLines),
    [overview, importedLines],
  );
  const canStartOpening = currentCard?.kind === 'Opening';
  const openingCard = canStartOpening ? currentCard : undefined;

  const handleGrade = (grade: ReviewGrade) => {
    const latency = Math.max(0, Math.round(performance.now() - startedAtRef.current));
    void submitGrade(grade, latency);
  };

  const handleBoardResult = (grade: ReviewGrade, latencyMs: number) => {
    void submitGrade(grade, latencyMs);
  };

  return (
    <Routes>
      <Route path="/" element={<Navigate to="/dashboard" replace />} />
      <Route
        path="/dashboard"
        element={
          <DashboardPage
            overview={dashboardOverview}
            openingPath="/review/opening"
            canStartOpening={canStartOpening}
            onImportLine={onImportLine}
            commandDispatcher={commandDispatcher}
          />
        }
      />
      <Route
        path="/review/opening"
        element={
          <OpeningReviewPage
            card={openingCard}
            onGrade={handleGrade}
            onBoardResult={handleBoardResult}
            backPath="/dashboard"
          />
        }
      />
      <Route path="/tools/board" element={<BlankBoardPage />} />
      <Route path="*" element={<Navigate to="/dashboard" replace />} />
    </Routes>
  );
};

const isTextEntryTarget = (target: EventTarget | null): boolean => {
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  const tagName = target.tagName;
  if (target.isContentEditable) {
    return true;
  }

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
  const dispatcher = useMemo(
    () =>
      createCommandDispatcher({
        onUnknownCommand: (input) => window.alert(input),
      }),
    [],
  );

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const isColonKey = isColonKeyPressed(event);

      if (isColonKey && !event.metaKey && !event.ctrlKey && !event.altKey) {
        if (isTextEntryTarget(event.target)) {
          return;
        }

        event.preventDefault();
        setIsConsoleOpen(true);
        return;
      }

      if (event.key === 'Escape') {
        if (isConsoleOpen) {
          event.preventDefault();
          setIsConsoleOpen(false);
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [isConsoleOpen]);

  const handleImportLine = (line: DetectedOpeningLine): ImportResult => {
    const existing = importedLines.find((candidate) => linesMatch(candidate, line));
    if (existing) {
      return { added: false, line: existing };
    }

    const nextLine = scheduleOpeningLine(line, importedLines.length);
    setImportedLines((previous) => [...previous, nextLine]);
    return { added: true, line: nextLine };
  };

  const handleOpenConsole = () => {
    setIsConsoleOpen(true);
  };
  const handleCloseConsole = () => {
    setIsConsoleOpen(false);
  };

  useEffect(() => {
    dispatcher.register('cb', () => {
      navigate('/tools/board');
    });
    dispatcher.register('db', () => {
      navigate('/dashboard');
    });
  }, [dispatcher, navigate]);

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
