import { useMemo } from 'react';

import './App.css';
import { ReviewDashboard } from './components/ReviewDashboard';
import { sampleSnapshot } from './fixtures/sampleSnapshot';
import { ReviewPlanner } from './services/ReviewPlanner';

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
