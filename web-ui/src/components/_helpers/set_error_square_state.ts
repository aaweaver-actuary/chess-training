import type { Square } from 'chess.js';
import type { ChessBoardElement } from 'chessboard-element';

const setErrorSquareState = (
  square: Square | null,
  setErrorSquare: (square: Square | null) => void,
  boardRef: React.RefObject<ChessBoardElement | null>,
) => {
  setErrorSquare(square);
  const board = boardRef.current;
  if (!board) {
    return;
  }

  if (square) {
    board.setAttribute('data-error-square', square);
  } else {
    board.removeAttribute('data-error-square');
  }
};

export default setErrorSquareState;
