import classNames from 'classnames';
import { Fragment, useMemo, useState } from 'react';
import type { CommitPayload, SyncChange, SyncChangeItem } from 'tauri-plugin-sync-api';
import { useChanges, useCreateCommit } from 'tauri-plugin-sync-api';
import { resolvedModelName } from '../lib/resolvedModelName';
import { Banner } from './core/Banner';
import { Button } from './core/Button';
import type { CheckboxProps } from './core/Checkbox';
import { Checkbox } from './core/Checkbox';
import { Editor } from './core/Editor';
import { InlineCode } from './core/InlineCode';
import { SplitLayout } from './core/SplitLayout';
import { HStack } from './core/Stacks';

interface TreeNode {
  children: TreeNode[];
  change: SyncChange;
  operation: 'added' | 'removed' | 'modified' | 'unmodified';
}

interface Props {
  workspaceId: string;
}

export function SyncCommitDialog({ workspaceId }: Props) {
  const [message, setMessage] = useState<string>('');
  const changes = useChanges(workspaceId, 'master');
  const createCommit = useCreateCommit(workspaceId);

  const [addedIds, setAddedIds] = useState<Record<string, boolean>>({});

  const tree: TreeNode | null = useMemo(() => {
    console.log(changes.data);
    const root = changes.data?.find(
      (c) => changeItemFromChange(c, addedIds).model.model_type === 'workspace',
    );
    if (root == null) {
      return null;
    }

    const buildNode = (parent: SyncChange): TreeNode => {
      const parentItem = changeItemFromChange(parent, addedIds);
      const children = (changes.data ?? [])
        .filter((c) => {
          const item = changeItemFromChange(c, addedIds);
          if (item.model.model_type === 'workspace') {
            return false; // Workspace will never be a child
          }

          if (item.model.model_type !== 'environment' && item.model.model.folderId != null) {
            return item.model.model.folderId === parentItem.model.model.id;
          }

          return item.model.model.workspaceId === parentItem.model.model.id;
        })
        .map((o) => buildNode(o));
      return {
        change: parent,
        children,
        operation: operationFromChange(parent),
      };
    };

    const tree = buildNode(root);
    return tree;
  }, [addedIds, changes.data]);

  const checkNode = (node: TreeNode, checked: boolean) => {
    setAddedIds((currentAddedIds) => {
      const newAddedIds = structuredClone(currentAddedIds);
      setCheckedOnChildren(node, newAddedIds, checked);
      return newAddedIds;
    });
  };

  const handleCreateCommit = async () => {
    if (tree == null) return;
    const changeItems = diffItemsForCommit(tree, addedIds);
    await createCommit.mutateAsync({ branch: 'master', message, changeItems });
  };

  if (tree == null) {
    return null;
  }

  return (
    <div className="grid grid-rows-1 h-full">
      <SplitLayout
        name="commit"
        layout="vertical"
        defaultRatio={0.3}
        firstSlot={({ style }) => (
          <div style={style} className="h-full overflow-y-auto -ml-1">
            <TreeNodeChildren node={tree} depth={0} onCheck={checkNode} addedIds={addedIds} />
          </div>
        )}
        secondSlot={({ style }) => (
          <div style={style} className="grid grid-rows-[minmax(0,1fr)_auto] gap-3 pb-2">
            <div className="bg-surface-highlight border border-border rounded-md overflow-hidden">
              <Editor
                className="!text-base font-sans h-full rounded-md"
                placeholder="Commit message..."
                onChange={setMessage}
              />
            </div>
            {createCommit.error && <Banner color="danger">{createCommit.error}</Banner>}
            <HStack justifyContent="end" space={2}>
              <Button color="secondary" size="sm" onClick={handleCreateCommit}>
                Commit
              </Button>
              <Button color="secondary" size="sm">
                Commit and Push
              </Button>
            </HStack>
          </div>
        )}
      />
    </div>
  );
}

function TreeNodeChildren({
  node,
  depth,
  addedIds,
  onCheck,
}: {
  node: TreeNode | null;
  depth: number;
  addedIds: Record<string, boolean>;
  onCheck: (node: TreeNode, checked: boolean) => void;
}) {
  if (node === null) return null;

  const checked = nodeCheckedStatus(node, addedIds);
  return (
    <div
      className={classNames(
        depth > 0 && 'pl-1 ml-[10px] border-l border-dashed border-border-subtle',
      )}
    >
      <div className="flex gap-3 w-full h-xs">
        <Checkbox
          className="w-full hover:bg-surface-highlight rounded px-1 group"
          checked={checked}
          title={
            <div className="flex items-center gap-1 w-full">
              <div>
                {resolvedModelName(changeItemFromChange(node.change, addedIds).model.model)}
              </div>
              <InlineCode
                className={classNames(
                  'py-0 ml-auto !bg-surface',
                  node.operation === 'unmodified' && 'text-secondary',
                  node.operation === 'modified' && 'text-info',
                  node.operation === 'added' && 'text-success',
                  node.operation === 'removed' && 'text-danger',
                )}
              >
                {node.operation}
              </InlineCode>
            </div>
          }
          onChange={(checked) => onCheck(node, checked)}
        />
      </div>

      {node.children.map((childNode) => {
        return (
          <Fragment key={idFromChange(childNode.change, addedIds)}>
            <TreeNodeChildren
              node={childNode}
              depth={depth + 1}
              onCheck={onCheck}
              addedIds={addedIds}
            />
          </Fragment>
        );
      })}
    </div>
  );
}

function nodeCheckedStatus(
  root: TreeNode,
  addedIds: Record<string, boolean>,
): CheckboxProps['checked'] {
  let leavesVisited = 0;
  let leavesChecked = 0;
  if (root.children.length === 0) {
    return addedIds[idFromChange(root, addedIds)] ?? false;
  }

  const visitChildren = (n: TreeNode) => {
    if (n.children.length === 0) {
      leavesVisited += 1;
      const checked = addedIds[idFromChange(n, addedIds)] ?? false;
      if (checked) leavesChecked += 1;
    }
    for (const child of n.children) {
      visitChildren(child);
    }
  };

  visitChildren(root);

  if (leavesVisited === leavesChecked) {
    return true;
  } else if (leavesChecked === 0) {
    return false;
  } else {
    return 'indeterminate';
  }
}

function setCheckedOnChildren(node: TreeNode, addedIds: Record<string, boolean>, checked: boolean) {
  const id = idFromChange(node, addedIds);

  if (node.children.length === 0) {
    addedIds[id] = checked;
  }

  for (const child of node.children) {
    setCheckedOnChildren(child, addedIds, checked);
  }
}

function diffItemsForCommit(
  root: TreeNode,
  addedIds: Record<string, boolean>,
): CommitPayload['changeItems'] {
  const changes: CommitPayload['changeItems'] = [];
  for (const child of root.children) {
    const wasAdded = !!addedIds[idFromChange(child, addedIds)];
    if (wasAdded) {
      changes.push(changeItemFromChange(child, addedIds));
    }

    changes.push(...diffItemsForCommit(child, addedIds));
  }

  // If children had IDs to commit, also add this node
  if (changes.length > 0) changes.unshift(changeItemFromChange(root, addedIds));

  return changes;
}

function changeItemFromChange(
  c: SyncChange | TreeNode,
  addedIds: Record<string, boolean>,
): SyncChangeItem {
  c = 'change' in c ? c.change : c;

  const v = c.next ?? c.prev;
  if (v == null) {
    // Should never happen
    throw new Error("Change didn't contain next or prev");
  }

  const isAdded = addedIds[v.model.model.id];
  if (c.prev != null && c.next == null) return c.prev;
  if (c.prev == null && c.next != null) return c.next;
  if (c.prev != null && c.next != null && c.prev.hash !== c.next.hash)
    return isAdded ? c.next : c.prev;

  return v;
}

function idFromChange(c: SyncChange | TreeNode, addedIds: Record<string, boolean>): string {
  return changeItemFromChange(c, addedIds).model.model.id;
}

function operationFromChange(c: SyncChange): TreeNode['operation'] {
  if (c.prev != null && c.next == null) return 'removed';
  if (c.prev == null && c.next != null) return 'added';
  if (c.prev != null && c.next != null && c.prev.hash !== c.next.hash) return 'modified';
  return 'unmodified';
}