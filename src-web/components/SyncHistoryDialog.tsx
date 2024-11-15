import { useCommits } from 'tauri-plugin-sync-api';

interface Props {
  workspaceId: string;
  hide: () => void;
}

export function SyncHistoryDialog({ workspaceId }: Props) {
  const commits = useCommits({ workspaceId, branch: 'master' });

  return (
    <table className="w-full text-sm mb-auto min-w-full max-w-full divide-y divide-surface-highlight">
      <thead>
        <tr>
          <th className="py-2 text-left">Date</th>
          <th className="py-2 text-left pl-4">Message</th>
        </tr>
      </thead>
      <tbody className="divide-y divide-surface-highlight">
        {commits.data?.map((c) => (
          <tr key={c.id}>
            <td className="py-2 select-text cursor-text font-mono font-semibold max-w-0">
              {c.createdAt}
            </td>
            <td className="py-2 pl-4 select-text cursor-text font-mono text-text-subtle whitespace-nowrap overflow-x-auto max-w-[200px] hide-scrollbars">
              {c.message ?? 'n/a'}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
