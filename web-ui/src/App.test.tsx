import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import App from './App';

describe('App', () => {
  it('renders the review dashboard with sample data', () => {
    render(<App />);

    expect(screen.getByRole('heading', { name: /Daily Review Summary/i })).toBeInTheDocument();
    expect(screen.getByText(/Upcoming Unlocks/i)).toBeInTheDocument();
  });
});
