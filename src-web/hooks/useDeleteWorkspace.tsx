import { useMutation, useQueryClient } from '@tanstack/react-query';
import { InlineCode } from '../components/core/InlineCode';
import { trackEvent } from '../lib/analytics';
import type { Workspace } from '@yaakapp/api';
import { invokeCmd } from '../lib/tauri';
import { useActiveWorkspaceId } from './useActiveWorkspaceId';
import { useAppRoutes } from './useAppRoutes';
import { useConfirm } from './useConfirm';
import { httpRequestsQueryKey } from './useHttpRequests';
import { workspacesQueryKey } from './useWorkspaces';

export function useDeleteWorkspace(workspace: Workspace | null) {
  const queryClient = useQueryClient();
  const activeWorkspaceId = useActiveWorkspaceId();
  const routes = useAppRoutes();
  const confirm = useConfirm();

  return useMutation<Workspace | null, string>({
    mutationKey: ['delete_workspace', workspace?.id],
    mutationFn: async () => {
      const confirmed = await confirm({
        id: 'delete-workspace',
        title: 'Delete Workspace',
        variant: 'delete',
        description: (
          <>
            Permanently delete <InlineCode>{workspace?.name}</InlineCode>?
          </>
        ),
      });
      if (!confirmed) return null;
      return invokeCmd('cmd_delete_workspace', { workspaceId: workspace?.id });
    },
    onSettled: () => trackEvent('workspace', 'delete'),
    onSuccess: async (workspace) => {
      if (workspace === null) return;

      const { id: workspaceId } = workspace;
      queryClient.setQueryData<Workspace[]>(workspacesQueryKey({}), (workspaces) =>
        workspaces?.filter((workspace) => workspace.id !== workspaceId),
      );
      if (workspaceId === activeWorkspaceId) {
        routes.navigate('workspaces');
      }

      // Also clean up other things that may have been deleted
      queryClient.setQueryData(httpRequestsQueryKey({ workspaceId }), []);
      await queryClient.invalidateQueries({ queryKey: httpRequestsQueryKey({ workspaceId }) });
    },
  });
}
