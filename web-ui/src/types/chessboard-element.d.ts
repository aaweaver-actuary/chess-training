/* eslint-disable @typescript-eslint/consistent-type-definitions */
import type { DetailedHTMLProps, HTMLAttributes, Ref } from 'react';
import type { ChessBoardElement } from 'chessboard-element/lib/chessboard-element';

type ChessBoardElementProps = Omit<
  DetailedHTMLProps<HTMLAttributes<ChessBoardElement>, ChessBoardElement>,
  'ref'
> & {
  position?: string;
  ref?: Ref<ChessBoardElement | null>;
};

declare module 'react' {
  namespace JSX {
    interface IntrinsicElements {
      'chess-board': ChessBoardElementProps;
    }
  }
}
