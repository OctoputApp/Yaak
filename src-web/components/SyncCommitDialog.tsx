import classNames from 'classnames';
import { Fragment, useState } from 'react';
import type { StageTreeNode } from 'tauri-plugin-sync-api';
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
import { EmptyStateText } from './EmptyStateText';

interface Props {
  workspaceId: string;
  hide: () => void;
}

export function SyncCommitDialog({ workspaceId, hide }: Props) {
  const tree = useChanges({ workspaceId, branch: 'master' });

  if (tree.isFetching || tree.data == null) {
    return null;
  }

  return <SyncCommitChanges tree={tree.data} workspaceId={workspaceId} onDone={hide} />;
}

function SyncCommitChanges({
  tree,
  workspaceId,
  onDone,
}: {
  tree: StageTreeNode;
  workspaceId: string;
  onDone: () => void;
}) {
  const createCommit = useCreateCommit({ workspaceId, branch: 'master' });
  const [message, setMessage] = useState<string>('');
  const [addedIds, setAddedIds] = useState<Record<string, boolean>>({});

  const checkNode = (node: StageTreeNode, checked: boolean) => {
    setAddedIds((currentAddedIds) => {
      const newAddedIds = structuredClone(currentAddedIds);
      setCheckedOnChildren(node, newAddedIds, checked);
      return newAddedIds;
    });
  };

  const handleCreateCommit = async () => {
    if (tree == null) {
      return;
    }

    await createCommit.mutateAsync({
      message,
      addedIds: Object.entries(addedIds)
        .map(([id, added]) => (added ? id : null))
        .filter((id) => id !== null),
    });
    onDone();
  };

  const hasAddedAnything = Object.values(addedIds).some((a) => a);

  if (tree == null) {
    return null;
  }

  if (!isNodeRelevant(tree)) {
    return (
      <EmptyStateText>
        No changes to commit.
        <br />
        Please check back once you have made changes.
      </EmptyStateText>
    );
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
              <Button
                color="secondary"
                size="sm"
                onClick={handleCreateCommit}
                disabled={!hasAddedAnything}
              >
                Commit
              </Button>
              {/*<Button color="secondary" size="sm" disabled={!hasAddedAnything}>*/}
              {/*  Commit and Push*/}
              {/*</Button>*/}
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
  node: StageTreeNode | null;
  depth: number;
  addedIds: Record<string, boolean>;
  onCheck: (node: StageTreeNode, checked: boolean) => void;
}) {
  if (node === null) return null;
  if (!isNodeRelevant(node)) return null;

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
          onChange={(checked) => onCheck(node, checked)}
          title={
            <div className="grid grid-cols-[minmax(0,1fr)_auto] gap-1 w-full">
              <div className="truncate">{resolvedModelName(node.model)}</div>
              {node.status !== 'unmodified' && (
                <InlineCode
                  className={classNames(
                    'py-0 ml-auto !bg-surface',
                    node.status === 'modified' && 'text-info',
                    node.status === 'untracked' && 'text-success',
                    node.status === 'removed' && 'text-danger',
                  )}
                >
                  {node.status}
                </InlineCode>
              )}
            </div>
          }
        />
      </div>

      {node.children.map((childNode) => {
        return (
          <Fragment key={childNode.model.id}>
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
  root: StageTreeNode,
  addedIds: Record<string, boolean>,
): CheckboxProps['checked'] {
  let leavesVisited = 0;
  let leavesChecked = 0;

  const visitChildren = (n: StageTreeNode) => {
    if (n.children.length === 0) {
      leavesVisited += 1;
      if (addedIds[n.model.id]) {
        leavesChecked += 1;
      }
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

function setCheckedOnChildren(
  node: StageTreeNode,
  addedIds: Record<string, boolean>,
  checked: boolean,
) {
  addedIds[node.model.id] = checked;

  for (const child of node.children) {
    setCheckedOnChildren(child, addedIds, checked);
  }
}

function isNodeRelevant(node: StageTreeNode): boolean {
  if (node.status !== 'unmodified') {
    return true;
  }

  // Recursively check children
  return node.children.some((c) => isNodeRelevant(c));
}
