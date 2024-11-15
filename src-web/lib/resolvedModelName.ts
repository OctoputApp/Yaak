import type { AnyModel, GrpcRequest, HttpRequest } from '@yaakapp-internal/models';

export function resolvedModelName(r: HttpRequest | GrpcRequest | AnyModel | null): string {
  if (r == null) return '';

  if (r.model !== 'grpc_request' && r.model !== 'http_request') {
    return 'name' in r ? r.name : '';
  }

  // Return name if it has one
  if (r.name) {
    return r.name;
  }

  // Replace variable syntax with variable name
  const withoutVariables = r.url.replace(/\$\{\[\s*([^\]\s]+)\s*]}/g, '$1');
  if (withoutVariables.trim() === '') {
    return r.model === 'http_request' ? 'New HTTP Request' : 'new gRPC Request';
  }

  // GRPC gets nice short names
  if (r.model === 'grpc_request' && r.service != null && r.method != null) {
    const shortService = r.service.split('.').pop();
    return `${shortService}/${r.method}`;
  }

  // Strip unnecessary protocol
  const withoutProto = withoutVariables.replace(/^https?:\/\//, '');

  return withoutProto;
}
