import { useMutation } from '@tanstack/react-query';
import type { HttpRequest } from '@yaakapp/api';
import { getHttpRequest } from '../lib/store';
import { invokeCmd } from '../lib/tauri';

export function useUpdateAnyHttpRequest() {
  return useMutation<
    void,
    unknown,
    { id: string; update: Partial<HttpRequest> | ((r: HttpRequest) => HttpRequest) }
  >({
    mutationKey: ['update_any_http_request'],
    mutationFn: async ({ id, update }) => {
      const request = await getHttpRequest(id);
      if (request === null) {
        throw new Error("Can't update a null request");
      }

      const patchedRequest =
        typeof update === 'function' ? update(request) : { ...request, ...update };
      await invokeCmd('cmd_update_http_request', { request: patchedRequest });
    },
  });
}
