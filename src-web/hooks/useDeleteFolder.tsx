import { useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { InlineCode } from '../components/core/InlineCode';
import { trackEvent } from '../lib/analytics';
import type { Folder } from '../lib/models';
import { getFolder } from '../lib/store';
import { useConfirm } from './useConfirm';
import { foldersQueryKey } from './useFolders';
import { httpRequestsQueryKey } from './useHttpRequests';

export function useDeleteFolder(id: string | null) {
  const queryClient = useQueryClient();
  const confirm = useConfirm();

  return useMutation<Folder | null, string>({
    mutationFn: async () => {
      const folder = await getFolder(id);
      const confirmed = await confirm({
        id: 'delete-folder',
        title: 'Delete Folder',
        variant: 'delete',
        description: (
          <>
            Permanently delete <InlineCode>{folder?.name}</InlineCode> and everything in it?
          </>
        ),
      });
      if (!confirmed) return null;
      return invoke('cmd_delete_folder', { folderId: id });
    },
    onSettled: () => trackEvent('folder', 'delete'),
    onSuccess: async (folder) => {
      // Was it cancelled?
      if (folder === null) return;

      const { workspaceId } = folder;

      // Nesting makes it hard to clean things up, so just clear everything that could have been deleted
      await queryClient.invalidateQueries(httpRequestsQueryKey({ workspaceId }));
      await queryClient.invalidateQueries(foldersQueryKey({ workspaceId }));
    },
  });
}
