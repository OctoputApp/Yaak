import classNames from 'classnames';
import type { HTMLAttributes, ReactElement, ReactNode } from 'react';

export function KeyValueRows({
  children,
}: {
  children:
    | ReactElement<HTMLAttributes<HTMLTableColElement>>
    | ReactElement<HTMLAttributes<HTMLTableColElement>>[];
}) {
  children = Array.isArray(children) ? children : [children];
  return (
    <table className="text-xs font-mono min-w-0 w-full mb-auto">
      <tbody className="divide-highlightSecondary">
        {children.map((child, i) => (
          <tr key={i}>{child}</tr>
        ))}
      </tbody>
    </table>
  );
}

interface Props {
  label: ReactNode;
  value: ReactNode;
  labelClassName?: string;
  labelColor?: 'secondary' | 'primary' | 'info';
}

export function KeyValueRow({ label, value, labelColor = 'secondary', labelClassName }: Props) {
  return (
    <>
      <td
        className={classNames(
          'py-0.5 pr-2 select-text cursor-text',
          labelClassName,
          labelColor === 'primary' && 'text-primary',
          labelColor === 'secondary' && 'text-text-subtle',
          labelColor === 'info' && 'text-info',
        )}
      >
        {label}
      </td>
      <td className="py-0.5 cursor-text select-text break-all min-w-0">{value}</td>
    </>
  );
}
