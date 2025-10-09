import { useEffect, useRef } from 'react';
import type { FC } from 'react';
import { Link } from 'react-router-dom';

import 'chessboard-element';

type BlankBoardPageProps = {
  backPath: string;
};

export const BlankBoardPage: FC<BlankBoardPageProps> = ({ backPath }) => {
  const boardRef = useRef<(HTMLElement & { position?: string }) | null>(null);

  useEffect(() => {
    const board = boardRef.current;
    if (!board) {
      return;
    }

    board.setAttribute('position', 'start');
    board.setAttribute('data-initial-position', 'start');
    board.position = 'start';
  }, []);

  return (
    <main className="app-shell blank-board-page">
      <nav aria-label="Page navigation" className="review-navigation">
        <Link to={backPath} className="nav-link floating-action">
          Back to Dashboard
        </Link>
      </nav>
      <section aria-label="Sandbox board" className="blank-board">
        <h1>Sandbox Board</h1>
        <chess-board
          ref={boardRef}
          data-testid="sandbox-board"
          position="start"
          style={{ width: 'min(90vw, 560px)' }}
        />
      </section>
    </main>
  );
};
