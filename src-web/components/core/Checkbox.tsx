import classNames from 'classnames';
import { Icon } from './Icon';
import { HStack } from './Stacks';

interface Props {
  checked: boolean;
  title: string;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  className?: string;
  inputWrapperClassName?: string;
  indeterminate?: boolean;
  hideLabel?: boolean;
}

export function Checkbox({
  checked,
  indeterminate,
  onChange,
  className,
  inputWrapperClassName,
  disabled,
  title,
  hideLabel,
}: Props) {
  return (
    <HStack
      as="label"
      space={2}
      alignItems="center"
      className={classNames(className, 'text-fg text-sm', disabled && 'opacity-disabled')}
    >
      <div className={classNames(inputWrapperClassName, 'relative flex')}>
        <input
          aria-hidden
          className={classNames(
            'appearance-none w-4 h-4 flex-shrink-0 border border-background-highlight-secondary',
            'rounded hocus:border-border-focus hocus:bg-focus/[5%] outline-none ring-0',
          )}
          type="checkbox"
          disabled={disabled}
          onChange={() => onChange(!checked)}
        />
        <div className="absolute inset-0 flex items-center justify-center">
          <Icon size="sm" icon={indeterminate ? 'minus' : checked ? 'check' : 'empty'} />
        </div>
      </div>
      {!hideLabel && title}
    </HStack>
  );
}
