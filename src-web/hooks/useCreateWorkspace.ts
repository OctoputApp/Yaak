import { useMutation } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { Workspace } from '../lib/models';
import { useAppRoutes } from './useAppRoutes';
import { usePrompt } from './usePrompt';

export function useCreateWorkspace() {
  const routes = useAppRoutes();
  const prompt = usePrompt();
  return useMutation<Workspace, void, void>({
    mutationFn: async () => {
      const name = await prompt({
        id: 'new-workspace',
        name: 'name',
        label: 'Name',
        defaultValue: 'My Workspace',
        title: 'New Workspace',
        confirmLabel: 'Create',
        placeholder: 'My Workspace',
      });
      return invoke('cmd_create_workspace', { name });
    },
    onSuccess: async (workspace) => {
      routes.navigate('workspace', { workspaceId: workspace.id });
    },
  });
}
