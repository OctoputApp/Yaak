import { useMutation, useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { CommitPayload } from './bindings/commands';
import { SyncCommit } from './bindings/models';
import { StageTreeNode } from './bindings/sync';

export * from './bindings/commands';
export * from './bindings/models';
export * from './bindings/sync';

export function useChanges({ workspaceId, branch }: { workspaceId: string; branch: string }) {
  return useQuery<StageTreeNode>({
    refetchOnMount: true,
    queryKey: ['sync.changes', workspaceId, branch],
    queryFn: () => invoke('plugin:sync|changes', { payload: { workspaceId, branch } }),
  });
}

export function useCommits({ workspaceId, branch }: { workspaceId: string; branch: string }) {
  return useQuery<SyncCommit[]>({
    refetchOnMount: true,
    queryKey: ['sync.commits', workspaceId, branch],
    queryFn: () => invoke('plugin:sync|commits', { payload: { workspaceId, branch } }),
  });
}

export function useCreateCommit({ workspaceId, branch }: { workspaceId: string; branch: string }) {
  return useMutation<void, String, Omit<CommitPayload, 'workspaceId' | 'branch'>>({
    mutationKey: ['sync.commit', workspaceId, branch],
    mutationFn: (payload) =>
      invoke('plugin:sync|commit', { payload: { ...payload, branch, workspaceId } }),
  });
}

export function usePush(workspaceId: string, branch: string) {
  return useMutation<void, String, Omit<CommitPayload, 'workspaceId' | 'branch'>>({
    mutationKey: ['sync.push', workspaceId, branch],
    mutationFn: (payload) =>
      invoke('plugin:sync|commit', { payload: { ...payload, branch, workspaceId } }),
  });
}
