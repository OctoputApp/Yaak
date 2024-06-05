import classNames from 'classnames';
import type { ReactNode } from 'react';

interface Props {
  children: ReactNode;
}

export function FormattedError({ children }: Props) {
  return (
    <pre
      className={classNames(
        'font-mono text-sm w-full select-auto cursor-text bg-background-highlight-secondary p-3 rounded',
        'whitespace-pre-wrap border border-fg-danger border-dashed overflow-x-auto',
      )}
    >
      {children}
    </pre>
  );
}
