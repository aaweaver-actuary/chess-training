import type { DetailedHTMLProps, HTMLAttributes } from 'react';

type ChessBoardElementProps = DetailedHTMLProps<HTMLAttributes<HTMLElement>, HTMLElement> & {
  position?: string;
};

declare module 'react/jsx-runtime' {
  namespace JSX {
    interface IntrinsicElements {
      'chess-board': ChessBoardElementProps;
    }
  }
}
