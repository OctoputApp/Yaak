import classNames from 'classnames';
import type { ReactNode } from 'react';
import React from 'react';

interface Props {
  children: ReactNode;
  className?: string;
}

export function EmptyStateText({ children, className }: Props) {
  return (
    <div
      className={classNames(
        className,
        'rounded-lg border border-dashed border-border-subtle',
        'h-full py-2 text-text-subtlest flex items-center text-center justify-center italic',
      )}
    >
      {children}
    </div>
  );
}
