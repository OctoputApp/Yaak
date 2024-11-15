import { useMutation } from '@tanstack/react-query';
import type { GrpcRequest } from '@yaakapp-internal/models';
import {useSetAtom} from "jotai";
import { InlineCode } from '../components/core/InlineCode';
import { trackEvent } from '../lib/analytics';
import { resolvedModelName } from '../lib/resolvedModelName';
import { getGrpcRequest } from '../lib/store';
import { invokeCmd } from '../lib/tauri';
import { useConfirm } from './useConfirm';
import {grpcRequestsAtom} from "./useGrpcRequests";
import {removeModelById} from "./useSyncModelStores";

export function useDeleteAnyGrpcRequest() {
  const confirm = useConfirm();
  const setGrpcRequests = useSetAtom(grpcRequestsAtom);

  return useMutation<GrpcRequest | null, string, string>({
    mutationKey: ['delete_any_grpc_request'],
    mutationFn: async (id) => {
      const request = await getGrpcRequest(id);
      if (request == null) return null;

      const confirmed = await confirm({
        id: 'delete-grpc-request',
        title: 'Delete Request',
        variant: 'delete',
        description: (
          <>
            Permanently delete <InlineCode>{resolvedModelName(request)}</InlineCode>?
          </>
        ),
      });
      if (!confirmed) return null;
      return invokeCmd('cmd_delete_grpc_request', { requestId: id });
    },
    onSuccess: (request) => {
      if (request == null) return;

      // Optimistic update
      setGrpcRequests(removeModelById(request));
    },
    onSettled: () => trackEvent('grpc_request', 'delete'),
  });
}
