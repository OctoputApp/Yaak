import { useMutation, useQueryClient } from '@tanstack/react-query';
import { InlineCode } from '../components/core/InlineCode';
import { trackEvent } from '../lib/analytics';
import type { Folder } from '@yaakapp/api';
import { getFolder } from '../lib/store';
import { invokeCmd } from '../lib/tauri';
import { useConfirm } from './useConfirm';
import { foldersQueryKey } from './useFolders';
import { httpRequestsQueryKey } from './useHttpRequests';

export function useDeleteFolder(id: string | null) {
  const queryClient = useQueryClient();
  const confirm = useConfirm();

  return useMutation<Folder | null, string>({
    mutationKey: ['delete_folder', id],
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
      return invokeCmd('cmd_delete_folder', { folderId: id });
    },
    onSettled: () => trackEvent('folder', 'delete'),
    onSuccess: async (folder) => {
      // Was it cancelled?
      if (folder === null) return;

      const { workspaceId } = folder;

      // Nesting makes it hard to clean things up, so just clear everything that could have been deleted
      await queryClient.invalidateQueries({ queryKey: httpRequestsQueryKey({ workspaceId }) });
      await queryClient.invalidateQueries({ queryKey: foldersQueryKey({ workspaceId }) });
    },
  });
}
