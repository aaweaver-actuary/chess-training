import { useEffect, useMemo, useRef, useSyncExternalStore } from 'react';

import './App.css';
import { ReviewDashboard } from './components/ReviewDashboard';
import { sampleSnapshot } from './fixtures/sampleSnapshot';
import { ReviewPlanner } from './services/ReviewPlanner';
import type { ReviewGrade } from './types/gateway';
import { sessionStore } from './state/sessionStore';

const planner = new ReviewPlanner();

const gradeLabels: ReviewGrade[] = ['Again', 'Hard', 'Good', 'Easy'];

function useSessionState() {
  return useSyncExternalStore(sessionStore.subscribe, sessionStore.getState, sessionStore.getState);
}

function App(): JSX.Element {
  const session = useSessionState();
  const { stats, currentCard, start, submitGrade } = session;
  const startedAtRef = useRef<number>(performance.now());

  const baselineOverview = useMemo(() => planner.buildOverview(sampleSnapshot), []);
  const overview = useMemo(() => {
    if (!stats) {
      return baselineOverview;
    }

    const totalDue = stats.due_count;
    const completed = stats.completed_count;
    const remaining = Math.max(totalDue - completed, 0);
    const completionRate = totalDue === 0 ? 1 : completed / totalDue;

    return {
      ...baselineOverview,
      progress: {
        ...baselineOverview.progress,
        totalDue,
        completedToday: completed,
        remaining,
        completionRate,
        accuracyRate: stats.accuracy,
      },
    };
  }, [baselineOverview, stats]);

  useEffect(() => {
    void start('demo-user');
  }, [start]);

  useEffect(() => {
    if (currentCard) {
      startedAtRef.current = performance.now();
    }
  }, [currentCard]);

  const handleGrade = (grade: ReviewGrade) => {
    const latency = Math.max(0, Math.round(performance.now() - startedAtRef.current));
    void submitGrade(grade, latency);
  };

  return (
    <main className="app-shell">
      <ReviewDashboard overview={overview} />
      <section aria-label="Review controls" className="review-controls">
        <h2>Grade Current Card</h2>
        <div className="grade-buttons">
          {gradeLabels.map((grade) => (
            <button
              key={grade}
              type="button"
              onClick={() => {
                handleGrade(grade);
              }}
            >
              {grade}
            </button>
          ))}
        </div>
      </section>
    </main>
  );
}

export default App;
