import type { DetailedHTMLProps, HTMLAttributes } from 'react';

type ChessBoardElementProps = DetailedHTMLProps<HTMLAttributes<HTMLElement>, HTMLElement> & {
  position?: string;
};

declare module 'react/jsx-runtime' {
  namespace JSX {
    type IntrinsicElements = {
      'chess-board': ChessBoardElementProps;
    };
  }
}
