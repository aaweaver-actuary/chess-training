import { forwardRef } from 'react';
import type { ButtonHTMLAttributes } from 'react';

import './FloatingCornerButton.css';

export type FloatingCornerButtonProps = ButtonHTMLAttributes<HTMLButtonElement>;

export const FloatingCornerButton = forwardRef<HTMLButtonElement, FloatingCornerButtonProps>(
  ({ className, type = 'button', ...buttonProps }, ref) => {
    const combinedClassName = ['floating-corner-button', className].filter(Boolean).join(' ');

    return <button {...buttonProps} className={combinedClassName} ref={ref} type={type} />;
  },
);

FloatingCornerButton.displayName = 'FloatingCornerButton';
