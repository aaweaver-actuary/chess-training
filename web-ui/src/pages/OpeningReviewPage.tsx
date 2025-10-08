import type { FC } from 'react';
import { Link } from 'react-router-dom';

import { OpeningReviewBoard } from '../components/OpeningReviewBoard';
import type { CardSummary, ReviewGrade } from '../types/gateway';

const gradeLabels: ReviewGrade[] = ['Again', 'Hard', 'Good', 'Easy'];

type OpeningReviewPageProps = {
  card?: CardSummary;
  onGrade: (grade: ReviewGrade) => void;
  onBoardResult: (grade: ReviewGrade, latencyMs: number) => void;
  backPath: string;
};

const GradeButtons: FC<{ onSelect: (grade: ReviewGrade) => void }> = ({ onSelect }) => (
  <div className="grade-buttons">
    {gradeLabels.map((grade) => (
      <button
        key={grade}
        type="button"
        onClick={() => {
          onSelect(grade);
        }}
      >
        {grade}
      </button>
    ))}
  </div>
);

export const OpeningReviewPage: FC<OpeningReviewPageProps> = ({
  card,
  onGrade,
  onBoardResult,
  backPath,
}) => (
  <main className="app-shell opening-review-page">
    <nav aria-label="Page navigation" className="review-navigation">
      <Link to={backPath} className="nav-link">
        Back to Dashboard
      </Link>
    </nav>
    {card ? (
      <>
        <section aria-label="Opening review" className="opening-review">
          <OpeningReviewBoard card={card} onResult={onBoardResult} />
        </section>
        <section aria-label="Review controls" className="review-controls">
          <h2>Grade Current Card</h2>
          <GradeButtons onSelect={onGrade} />
        </section>
      </>
    ) : (
      <p className="empty-state">No opening card available right now.</p>
    )}
  </main>
);
