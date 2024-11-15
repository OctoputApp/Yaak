import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useEffect } from 'react';
import { resolvedModelName } from '../lib/resolvedModelName';
import { useActiveEnvironment } from './useActiveEnvironment';
import { useActiveRequest } from './useActiveRequest';
import { useActiveWorkspace } from './useActiveWorkspace';
import { useAppInfo } from './useAppInfo';
import { useOsInfo } from './useOsInfo';
import { emit } from '@tauri-apps/api/event';

export function useSyncWorkspaceRequestTitle() {
  const activeRequest = useActiveRequest();
  const activeWorkspace = useActiveWorkspace();
  const [activeEnvironment] = useActiveEnvironment();
  const osInfo = useOsInfo();
  const appInfo = useAppInfo();

  useEffect(() => {
    if (osInfo.osType == null) {
      return;
    }

    let newTitle = activeWorkspace ? activeWorkspace.name : 'Yaak';
    if (activeEnvironment) {
      newTitle += ` [${activeEnvironment.name}]`;
    }
    if (activeRequest) {
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      newTitle += ` › ${resolvedModelName(activeRequest)}`;
    }

    if (appInfo.isDev) {
      newTitle = `[DEV] ${newTitle}`;
    }

    // TODO: This resets the stoplight position so we can't use it on macOS yet. So we send
    //   a custom command instead
    if (osInfo.osType !== 'macos') {
      getCurrentWebviewWindow().setTitle(newTitle).catch(console.error);
    } else {
      emit('yaak_title_changed', newTitle).catch(console.error);
    }
  }, [activeEnvironment, activeRequest, activeWorkspace, appInfo.isDev, osInfo.osType]);
}
