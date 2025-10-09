import { useEffect, useMemo, useRef, useState, useSyncExternalStore } from 'react';
import { Navigate, Route, Routes } from 'react-router-dom';

import './App.css';
import { sampleSnapshot } from './fixtures/sampleSnapshot';
import { DashboardPage } from './pages/DashboardPage';
import { OpeningReviewPage } from './pages/OpeningReviewPage';
import { ReviewPlanner } from './services/ReviewPlanner';
import type { CardSummary, ReviewGrade } from './types/gateway';
import { sessionStore } from './state/sessionStore';
import { CommandConsole } from './components/CommandConsole';

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

const SessionRoutes = () => {
  const session = useSessionState();
  const { stats, currentCard, start, submitGrade } = session;
  useSessionLifecycle(start);

  const startedAtRef = useStartTimestamp(currentCard);

  const overview = useMemo(() => buildOverview(stats), [stats]);
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
            overview={overview}
            openingPath="/review/opening"
            canStartOpening={canStartOpening}
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

  const handleOpenConsole = () => {
    setIsConsoleOpen(true);
  };
  const handleCloseConsole = () => {
    setIsConsoleOpen(false);
  };

  return (
    <>
      <SessionRoutes />
      <CommandConsole
        isOpen={isConsoleOpen}
        onOpen={handleOpenConsole}
        onClose={handleCloseConsole}
      />
    </>
  );
};

export default App;
