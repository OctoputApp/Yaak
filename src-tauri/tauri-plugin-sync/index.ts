import { useMutation, useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { ChangesPayload, CommitPayload } from './bindings/commands';
import { SyncChange } from './bindings/sync';

export * from './bindings/commands';
export * from './bindings/models';
export * from './bindings/sync';

export function useChanges(workspaceId: string, branch: string) {
  return useQuery<SyncChange[]>({
    queryKey: ['sync.changes', workspaceId, branch],
    queryFn: () => {
      const payload: ChangesPayload = {
        workspaceId,
        branch,
      };
      return invoke('plugin:sync|changes', { payload });
    },
  });
}

export function useCreateCommit(workspaceId: string) {
  return useMutation<void, String, Omit<CommitPayload, 'workspaceId'>>({
    mutationKey: ['sync.commit', workspaceId],
    mutationFn: (payload) => invoke('plugin:sync|commit', { payload: { ...payload, workspaceId } }),
  });
}