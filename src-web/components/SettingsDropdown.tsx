import { open } from '@tauri-apps/plugin-shell';
import { useRef, useState } from 'react';
import { useAppInfo } from '../hooks/useAppInfo';
import { useCheckForUpdates } from '../hooks/useCheckForUpdates';
import { useExportData } from '../hooks/useExportData';
import { useImportData } from '../hooks/useImportData';
import { useListenToTauriEvent } from '../hooks/useListenToTauriEvent';
import type { DropdownRef } from './core/Dropdown';
import { Dropdown } from './core/Dropdown';
import { Icon } from './core/Icon';
import { IconButton } from './core/IconButton';
import { useDialog } from './DialogContext';
import { KeyboardShortcutsDialog } from './KeyboardShortcutsDialog';
import { SettingsDialog } from './SettingsDialog';

export function SettingsDropdown() {
  const importData = useImportData();
  const exportData = useExportData();
  const appInfo = useAppInfo();
  const dropdownRef = useRef<DropdownRef>(null);
  const dialog = useDialog();
  const checkForUpdates = useCheckForUpdates();
  const [showChangelog, setShowChangelog] = useState<boolean>(false);

  useListenToTauriEvent('show_changelog', () => {
    setShowChangelog(true);
  });

  return (
    <Dropdown
      ref={dropdownRef}
      onClose={() => setShowChangelog(false)}
      items={[
        {
          key: 'settings',
          label: 'Settings',
          hotKeyAction: 'settings.show',
          leftSlot: <Icon icon="settings" />,
          onSelect: () => {
            dialog.show({
              id: 'settings',
              size: 'md',
              title: 'Settings',
              render: () => <SettingsDialog />,
            });
          },
        },
        {
          key: 'hotkeys',
          label: 'Keyboard shortcuts',
          hotKeyAction: 'hotkeys.showHelp',
          leftSlot: <Icon icon="keyboard" />,
          onSelect: () => {
            dialog.show({
              id: 'hotkey',
              title: 'Keyboard Shortcuts',
              size: 'sm',
              render: () => <KeyboardShortcutsDialog />,
            });
          },
        },
        {
          key: 'import-data',
          label: 'Import Data',
          leftSlot: <Icon icon="folderInput" />,
          onSelect: () => importData.mutate(),
        },
        {
          key: 'export-data',
          label: 'Export Data',
          leftSlot: <Icon icon="folderOutput" />,
          onSelect: () => exportData.mutate(),
        },
        { type: 'separator', label: `Yaak v${appInfo.data?.version}` },
        {
          key: 'update-check',
          label: 'Check for Updates',
          leftSlot: <Icon icon="update" />,
          onSelect: () => checkForUpdates.mutate(),
        },
        {
          key: 'feedback',
          label: 'Feedback',
          leftSlot: <Icon icon="chat" />,
          rightSlot: <Icon icon="externalLink" />,
          onSelect: () => open('https://yaak.canny.io'),
        },
        {
          key: 'changelog',
          label: 'Changelog',
          variant: showChangelog ? 'notify' : 'default',
          leftSlot: <Icon icon="cake" />,
          rightSlot: <Icon icon="externalLink" />,
          onSelect: () => open(`https://yaak.app/changelog/${appInfo.data?.version}`),
        },
      ]}
    >
      <IconButton
        size="sm"
        title="Main Menu"
        icon="settings"
        className="pointer-events-auto"
        showBadge={showChangelog}
      />
    </Dropdown>
  );
}
