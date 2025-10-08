import { useEffect, useMemo, useRef, useSyncExternalStore } from 'react';
import { Navigate, Route, Routes } from 'react-router-dom';

import './App.css';
import { sampleSnapshot } from './fixtures/sampleSnapshot';
import { DashboardPage } from './pages/DashboardPage';
import { OpeningReviewPage } from './pages/OpeningReviewPage';
import { ReviewPlanner } from './services/ReviewPlanner';
import type { CardSummary, ReviewGrade } from './types/gateway';
import { sessionStore } from './state/sessionStore';

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

const App = (): JSX.Element => <SessionRoutes />;

export default App;
