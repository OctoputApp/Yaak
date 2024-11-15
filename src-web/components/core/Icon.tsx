import classNames from 'classnames';
import * as lucide from 'lucide-react';
import type { HTMLAttributes } from 'react';
import { memo } from 'react';

const icons = {
  alert_triangle: lucide.AlertTriangleIcon,
  archive: lucide.ArchiveIcon,
  arrow_big_down_dash: lucide.ArrowBigDownDashIcon,
  arrow_right_circle: lucide.ArrowRightCircleIcon,
  arrow_big_left_dash: lucide.ArrowBigLeftDashIcon,
  arrow_big_right: lucide.ArrowBigRightIcon,
  arrow_big_right_dash: lucide.ArrowBigRightDashIcon,
  arrow_big_up_dash: lucide.ArrowBigUpDashIcon,
  arrow_down: lucide.ArrowDownIcon,
  arrow_down_to_dot: lucide.ArrowDownToDotIcon,
  arrow_up: lucide.ArrowUpIcon,
  arrow_up_down: lucide.ArrowUpDownIcon,
  arrow_up_from_dot: lucide.ArrowUpFromDotIcon,
  badge_check: lucide.BadgeCheckIcon,
  box: lucide.BoxIcon,
  cake: lucide.CakeIcon,
  chat: lucide.MessageSquare,
  check: lucide.CheckIcon,
  check_circle: lucide.CheckCircleIcon,
  chevron_down: lucide.ChevronDownIcon,
  chevron_right: lucide.ChevronRightIcon,
  circle_alert: lucide.CircleAlertIcon,
  clock: lucide.ClockIcon,
  cloud: lucide.CloudIcon,
  code: lucide.CodeIcon,
  cookie: lucide.CookieIcon,
  copy: lucide.CopyIcon,
  copy_check: lucide.CopyCheck,
  download: lucide.DownloadIcon,
  external_link: lucide.ExternalLinkIcon,
  eye: lucide.EyeIcon,
  eye_closed: lucide.EyeOffIcon,
  file_code: lucide.FileCodeIcon,
  filter: lucide.FilterIcon,
  flask: lucide.FlaskConicalIcon,
  folder_input: lucide.FolderInputIcon,
  folder_output: lucide.FolderOutputIcon,
  git_branch: lucide.GitBranchIcon,
  git_commit: lucide.GitCommitIcon,
  git_commit_vertical: lucide.GitCommitVerticalIcon,
  git_pull_request: lucide.GitPullRequestIcon,
  git_fork: lucide.GitForkIcon,
  grip_vertical: lucide.GripVerticalIcon,
  hand: lucide.HandIcon,
  help: lucide.CircleHelpIcon,
  history: lucide.HistoryIcon,
  house: lucide.HomeIcon,
  info: lucide.InfoIcon,
  keyboard: lucide.KeyboardIcon,
  left_panel_hidden: lucide.PanelLeftOpenIcon,
  left_panel_visible: lucide.PanelLeftCloseIcon,
  magic_wand: lucide.Wand2Icon,
  minus: lucide.MinusIcon,
  moon: lucide.MoonIcon,
  more_vertical: lucide.MoreVerticalIcon,
  paste: lucide.ClipboardPasteIcon,
  pencil: lucide.PencilIcon,
  pin: lucide.PinIcon,
  plug: lucide.Plug,
  plus: lucide.PlusIcon,
  plus_circle: lucide.PlusCircleIcon,
  refresh: lucide.RefreshCwIcon,
  save: lucide.SaveIcon,
  search: lucide.SearchIcon,
  send_horizontal: lucide.SendHorizonalIcon,
  settings: lucide.SettingsIcon,
  sparkles: lucide.SparklesIcon,
  sun: lucide.SunIcon,
  table: lucide.TableIcon,
  text: lucide.FileTextIcon,
  trash: lucide.Trash2Icon,
  unpin: lucide.PinOffIcon,
  update: lucide.RefreshCcwIcon,
  upload: lucide.UploadIcon,
  x: lucide.XIcon,
  _unknown: lucide.ShieldAlertIcon,

  empty: (props: HTMLAttributes<HTMLSpanElement>) => <span {...props} />,
};

export interface IconProps {
  icon: keyof typeof icons;
  className?: string;
  size?: '2xs' | 'xs' | 'sm' | 'md' | 'lg';
  spin?: boolean;
  title?: string;
}

export const Icon = memo(function Icon({ icon, spin, size = 'md', className, title }: IconProps) {
  const Component = icons[icon] ?? icons._unknown;
  return (
    <Component
      title={title}
      className={classNames(
        className,
        'text-inherit flex-shrink-0',
        size === 'lg' && 'h-5 w-5',
        size === 'md' && 'h-4 w-4',
        size === 'sm' && 'h-3.5 w-3.5',
        size === 'xs' && 'h-3 w-3',
        size === '2xs' && 'h-2.5 w-2.5',
        spin && 'animate-spin',
      )}
    />
  );
});
