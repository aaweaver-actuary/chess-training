import { forwardRef } from 'react';
import type { ComponentPropsWithRef, ElementType, ReactElement } from 'react';

import './FloatingCornerButton.css';

export type FloatingCornerButtonPlacement =
  | 'top-right'
  | 'top-left'
  | 'bottom-right'
  | 'bottom-left';

export type FloatingCornerButtonStrategy = 'absolute' | 'fixed';

type FloatingCornerButtonBaseProps = {
  as?: ElementType;
  placement?: FloatingCornerButtonPlacement;
  strategy?: FloatingCornerButtonStrategy;
  isActive?: boolean;
  className?: string;
};

type PolymorphicProps<T extends ElementType> = FloatingCornerButtonBaseProps &
  Omit<ComponentPropsWithRef<T>, keyof FloatingCornerButtonBaseProps>;

type PolymorphicRef<T extends ElementType> = ComponentPropsWithRef<T>['ref'];

export type FloatingCornerButtonProps<T extends ElementType = 'button'> =
  PolymorphicProps<T>;

const FloatingCornerButtonInner = <T extends ElementType = 'button'>(
  {
    as,
    placement = 'top-right',
    strategy = 'absolute',
    isActive = false,
    className,
    ...restProps
  }: FloatingCornerButtonProps<T>,
  ref: PolymorphicRef<T>,
): ReactElement => {
  const Component = as ?? 'button';

  const combinedClassName = [
    'floating-corner-button',
    `floating-corner-button--${placement}`,
    `floating-corner-button--strategy-${strategy}`,
    className,
  ]
    .filter(Boolean)
    .join(' ');

  const dataState = isActive ? 'active' : undefined;

  if (Component === 'button') {
    const { type = 'button', ...buttonRest } = restProps as ComponentPropsWithRef<'button'>;

    return (
      <button
        {...buttonRest}
        className={combinedClassName}
        data-state={dataState}
        ref={ref as PolymorphicRef<'button'>}
        type={type}
      />
    );
  }

  return (
    <Component
      {...(restProps as ComponentPropsWithRef<T>)}
      className={combinedClassName}
      data-state={dataState}
      ref={ref}
    />
  );
};

type FloatingCornerButtonComponent = <T extends ElementType = 'button'>(
  props: FloatingCornerButtonProps<T> & { ref?: PolymorphicRef<T> },
) => ReactElement;

export const FloatingCornerButton = forwardRef(FloatingCornerButtonInner) as FloatingCornerButtonComponent;

FloatingCornerButton.displayName = 'FloatingCornerButton';
