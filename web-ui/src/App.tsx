import { useMemo } from 'react';

import './App.css';
import { ReviewDashboard } from './components/ReviewDashboard';
import { ReviewPlanner, type ReviewSnapshot } from './services/ReviewPlanner';

const sampleSnapshot: ReviewSnapshot = {
  dueCards: 18,
  completedCards: 11,
  accuracyRate: 0.86,
  streakLength: 7,
  upcomingUnlocks: [
    {
      id: 'unlock-italian',
      move: 'Bc4',
      idea: 'Pressure f7 in the Italian Game',
      scheduledFor: '2024-01-14',
    },
    {
      id: 'unlock-scandi',
      move: 'Qxd5',
      idea: 'Centralize the queen in the Scandinavian',
      scheduledFor: '2024-01-15',
    },
    {
      id: 'unlock-tactic',
      move: 'Nxf7+',
      idea: 'Tactic: Greek gift sacrifice',
      scheduledFor: '2024-01-16',
    },
  ],
};

const planner = new ReviewPlanner();

function App(): JSX.Element {
  const overview = useMemo(() => planner.buildOverview(sampleSnapshot), []);

  return (
    <main className="app-shell">
      <ReviewDashboard overview={overview} />
    </main>
  );
}

export default App;
