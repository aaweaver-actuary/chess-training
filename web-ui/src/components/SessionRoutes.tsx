import { useEffect, useMemo, useRef, useSyncExternalStore } from 'react';
import type { JSX } from 'react';
import { Navigate, Route, Routes } from 'react-router-dom';

import { baselineReviewOverview } from '../fixtures/baselineOverview';
import { BlankBoardPage } from '../pages/BlankBoardPage';
import { DashboardPage } from '../pages/DashboardPage';
import { OpeningReviewPage } from '../pages/OpeningReviewPage';
import type { ReviewOverview } from '../types/reviewOverview';
import type { CardSummary, ReviewGrade } from '../types/gateway';
import type { DetectedOpeningLine, ImportResult, ScheduledOpeningLine } from '../types/repertoire';
import { sessionStore } from '../state/sessionStore';
import { composeOverview, extendOverviewWithImports } from '../utils/dashboardOverview';
import type { CommandDispatcher } from '../utils/commandDispatcher';

const baselineOverview = baselineReviewOverview;

const useSessionState = () =>
  useSyncExternalStore(sessionStore.subscribe, sessionStore.getState, sessionStore.getState);

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

type SessionRoutesProps = {
  importedLines: ScheduledOpeningLine[];
  onImportLine: (line: DetectedOpeningLine) => ImportResult;
  commandDispatcher?: CommandDispatcher;
};

const enhanceOverview = (
  stats: ReturnType<typeof sessionStore.getState>['stats'],
  importedLines: ScheduledOpeningLine[],
): ReviewOverview => {
  const overview = composeOverview(baselineOverview, stats);
  return extendOverviewWithImports(overview, importedLines);
};

export const SessionRoutes = ({
  importedLines,
  onImportLine,
  commandDispatcher,
}: SessionRoutesProps): JSX.Element => {
  const session = useSessionState();
  const { stats, currentCard, start, submitGrade } = session;
  useSessionLifecycle(start);

  const startedAtRef = useStartTimestamp(currentCard);
  const overview = useMemo(() => enhanceOverview(stats, importedLines), [stats, importedLines]);
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

export type { SessionRoutesProps };
