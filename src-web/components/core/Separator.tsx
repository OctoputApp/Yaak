import classNames from 'classnames';
import type { ReactNode } from 'react';

interface Props {
  orientation?: 'horizontal' | 'vertical';
  variant?: 'primary' | 'secondary';
  className?: string;
  children?: ReactNode;
}

export function Separator({
  className,
  variant = 'primary',
  orientation = 'horizontal',
  children,
}: Props) {
  return (
    <div role="separator" className={classNames(className, 'flex items-center')}>
      {children && <div className="text-xs text-fg-subtler mr-2 whitespace-nowrap">{children}</div>}
      <div
        className={classNames(
          variant === 'primary' && 'bg-highlight',
          variant === 'secondary' && 'bg-highlightSecondary',
          orientation === 'horizontal' && 'w-full h-[1px]',
          orientation === 'vertical' && 'h-full w-[1px]',
        )}
      />
    </div>
  );
}
