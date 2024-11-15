import type {
  AnyModel,
  Folder,
  GrpcConnection,
  GrpcRequest,
  HttpRequest,
  HttpResponse,
  Workspace,
} from '@yaakapp-internal/models';
import classNames from 'classnames';
import type { ReactNode } from 'react';
import React, { Fragment, useCallback, useMemo, useRef, useState } from 'react';
import type { XYCoord } from 'react-dnd';
import { useDrag, useDrop } from 'react-dnd';
import { useKey, useKeyPressEvent } from 'react-use';
import { useActiveCookieJar } from '../hooks/useActiveCookieJar';
import { useActiveEnvironment } from '../hooks/useActiveEnvironment';

import { useActiveRequest } from '../hooks/useActiveRequest';
import { useActiveWorkspace } from '../hooks/useActiveWorkspace';
import { useAppRoutes } from '../hooks/useAppRoutes';
import { useCreateDropdownItems } from '../hooks/useCreateDropdownItems';
import { useDeleteFolder } from '../hooks/useDeleteFolder';
import { useDeleteRequest } from '../hooks/useDeleteRequest';
import { useDuplicateGrpcRequest } from '../hooks/useDuplicateGrpcRequest';
import { useDuplicateHttpRequest } from '../hooks/useDuplicateHttpRequest';
import { useFolders } from '../hooks/useFolders';
import { useGrpcConnections } from '../hooks/useGrpcConnections';
import { useHotKey } from '../hooks/useHotKey';
import type { CallableHttpRequestAction } from '../hooks/useHttpRequestActions';
import { useHttpRequestActions } from '../hooks/useHttpRequestActions';
import { useHttpResponses } from '../hooks/useHttpResponses';
import { useKeyValue } from '../hooks/useKeyValue';
import { useMoveToWorkspace } from '../hooks/useMoveToWorkspace';
import { usePrompt } from '../hooks/usePrompt';
import { useRenameRequest } from '../hooks/useRenameRequest';
import { useRequests } from '../hooks/useRequests';
import { useScrollIntoView } from '../hooks/useScrollIntoView';
import { useSendAnyHttpRequest } from '../hooks/useSendAnyHttpRequest';
import { useSendManyRequests } from '../hooks/useSendManyRequests';
import { useSidebarHidden } from '../hooks/useSidebarHidden';
import { useUpdateAnyFolder } from '../hooks/useUpdateAnyFolder';
import { useUpdateAnyGrpcRequest } from '../hooks/useUpdateAnyGrpcRequest';
import { useUpdateAnyHttpRequest } from '../hooks/useUpdateAnyHttpRequest';
import { useWorkspaces } from '../hooks/useWorkspaces';
import { resolvedModelName } from '../lib/resolvedModelName';
import { isResponseLoading } from '../lib/model_util';
import { getHttpRequest } from '../lib/store';
import type { DropdownItem } from './core/Dropdown';
import { ContextMenu } from './core/Dropdown';
import { HttpMethodTag } from './core/HttpMethodTag';
import { Icon } from './core/Icon';
import { InlineCode } from './core/InlineCode';
import { VStack } from './core/Stacks';
import { StatusTag } from './core/StatusTag';
import { DropMarker } from './DropMarker';
import { SyncDropdown } from './SyncDropdown';

interface Props {
  className?: string;
}

enum ItemTypes {
  REQUEST = 'request',
}

interface TreeNode {
  item: Workspace | Folder | HttpRequest | GrpcRequest;
  children: TreeNode[];
  depth: number;
}

export function Sidebar({ className }: Props) {
  const [hidden, setHidden] = useSidebarHidden();
  const sidebarRef = useRef<HTMLLIElement>(null);
  const activeRequest = useActiveRequest();
  const [activeEnvironment] = useActiveEnvironment();
  const [activeCookieJar] = useActiveCookieJar();
  const folders = useFolders();
  const requests = useRequests();
  const activeWorkspace = useActiveWorkspace();
  const httpRequestActions = useHttpRequestActions();
  const httpResponses = useHttpResponses();
  const grpcConnections = useGrpcConnections();
  const duplicateHttpRequest = useDuplicateHttpRequest({
    id: activeRequest?.id ?? null,
    navigateAfter: true,
  });
  const duplicateGrpcRequest = useDuplicateGrpcRequest({
    id: activeRequest?.id ?? null,
    navigateAfter: true,
  });
  const routes = useAppRoutes();
  const [hasFocus, setHasFocus] = useState<boolean>(false);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [selectedTree, setSelectedTree] = useState<TreeNode | null>(null);
  const updateAnyHttpRequest = useUpdateAnyHttpRequest();
  const updateAnyGrpcRequest = useUpdateAnyGrpcRequest();
  const updateAnyFolder = useUpdateAnyFolder();
  const [draggingId, setDraggingId] = useState<string | null>(null);
  const [hoveredTree, setHoveredTree] = useState<TreeNode | null>(null);
  const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);
  const collapsed = useKeyValue<Record<string, boolean>>({
    key: ['sidebar_collapsed', activeWorkspace?.id ?? 'n/a'],
    fallback: {},
    namespace: 'no_sync',
  });

  useHotKey('http_request.duplicate', async () => {
    if (activeRequest?.model === 'http_request') {
      await duplicateHttpRequest.mutateAsync();
    } else {
      await duplicateGrpcRequest.mutateAsync();
    }
  });

  const isCollapsed = useCallback(
    (id: string) => collapsed.value?.[id] ?? false,
    [collapsed.value],
  );

  const { tree, treeParentMap, selectableRequests } = useMemo<{
    tree: TreeNode | null;
    treeParentMap: Record<string, TreeNode>;
    selectedRequest: HttpRequest | GrpcRequest | null;
    selectableRequests: {
      id: string;
      index: number;
      tree: TreeNode;
    }[];
  }>(() => {
    const treeParentMap: Record<string, TreeNode> = {};
    const selectableRequests: {
      id: string;
      index: number;
      tree: TreeNode;
    }[] = [];

    if (activeWorkspace == null) {
      return { tree: null, treeParentMap, selectableRequests, selectedRequest: null };
    }

    let selectedRequest: HttpRequest | GrpcRequest | null = null;
    let selectableRequestIndex = 0;

    // Put requests and folders into a tree structure
    const next = (node: TreeNode): TreeNode => {
      if (
        node.item.id === selectedId &&
        (node.item.model === 'http_request' || node.item.model === 'grpc_request')
      ) {
        selectedRequest = node.item;
      }
      const childItems = [...requests, ...folders].filter((f) =>
        node.item.model === 'workspace' ? f.folderId == null : f.folderId === node.item.id,
      );

      // Recurse to children
      const isCollapsed = collapsed.value?.[node.item.id];
      const depth = node.depth + 1;
      childItems.sort((a, b) => a.sortPriority - b.sortPriority);
      for (const item of childItems) {
        treeParentMap[item.id] = node;
        // Add to children
        node.children.push(next({ item, children: [], depth }));
        // Add to selectable requests
        if (item.model !== 'folder' && !isCollapsed) {
          selectableRequests.push({ id: item.id, index: selectableRequestIndex++, tree: node });
        }
      }

      return node;
    };

    const tree = next({ item: activeWorkspace, children: [], depth: 0 });

    return { tree, treeParentMap, selectableRequests, selectedRequest };
  }, [activeWorkspace, selectedId, requests, folders, collapsed.value]);

  const focusActiveRequest = useCallback(
    (
      args: {
        forced?: {
          id: string;
          tree: TreeNode;
        };
        noFocusSidebar?: boolean;
      } = {},
    ) => {
      const { forced, noFocusSidebar } = args;
      const tree = forced?.tree ?? treeParentMap[activeRequest?.id ?? 'n/a'] ?? null;
      const children = tree?.children ?? [];
      const id =
        forced?.id ?? children.find((m) => m.item.id === activeRequest?.id)?.item.id ?? null;

      setHasFocus(true);
      setSelectedId(id);
      setSelectedTree(tree);

      if (id == null) {
        return;
      }
      if (!noFocusSidebar) {
        sidebarRef.current?.focus();
      }
    },
    [activeRequest, treeParentMap],
  );

  const handleSelect = useCallback(
    async (id: string, opts: { noFocus?: boolean } = {}) => {
      const tree = treeParentMap[id ?? 'n/a'] ?? null;
      const children = tree?.children ?? [];
      const node = children.find((m) => m.item.id === id) ?? null;
      if (node == null || tree == null || node.item.model === 'workspace') {
        return;
      }

      const { item } = node;

      if (item.model === 'folder') {
        await collapsed.set((c) => ({ ...c, [item.id]: !c[item.id] }));
      } else {
        routes.navigate('request', {
          requestId: id,
          workspaceId: item.workspaceId,
          environmentId: activeEnvironment?.id ?? null,
          cookieJarId: activeCookieJar?.id ?? null,
        });
        setSelectedId(id);
        setSelectedTree(tree);
        if (!opts.noFocus) focusActiveRequest({ forced: { id, tree } });
      }
    },
    [
      treeParentMap,
      collapsed,
      routes,
      activeEnvironment?.id,
      activeCookieJar?.id,
      focusActiveRequest,
    ],
  );

  const handleClearSelected = useCallback(() => {
    setSelectedId(null);
    setSelectedTree(null);
  }, []);

  const handleFocus = useCallback(() => {
    if (hasFocus) return;
    focusActiveRequest({ noFocusSidebar: true });
  }, [focusActiveRequest, hasFocus]);

  const handleBlur = useCallback(() => setHasFocus(false), []);

  useHotKey('sidebar.focus', async () => {
    // Hide the sidebar if it's already focused
    if (!hidden && hasFocus) {
      await setHidden(true);
      return;
    }

    // Show the sidebar if it's hidden
    if (hidden) {
      await setHidden(false);
    }

    // Select 0th index on focus if none selected
    focusActiveRequest(
      selectedTree != null && selectedId != null
        ? { forced: { id: selectedId, tree: selectedTree } }
        : undefined,
    );
  });

  useKeyPressEvent('Enter', (e) => {
    if (!hasFocus) return;
    const selected = selectableRequests.find((r) => r.id === selectedId);
    if (!selected || selected.id === activeRequest?.id || activeWorkspace == null) {
      return;
    }

    e.preventDefault();
    routes.navigate('request', {
      requestId: selected.id,
      workspaceId: activeWorkspace?.id ?? null,
      environmentId: activeEnvironment?.id ?? null,
      cookieJarId: activeCookieJar?.id ?? null,
    });
  });

  useKey(
    'ArrowUp',
    (e) => {
      if (!hasFocus) return;
      e.preventDefault();
      const i = selectableRequests.findIndex((r) => r.id === selectedId);
      const newSelectable = selectableRequests[i - 1];
      if (newSelectable == null) {
        return;
      }

      setSelectedId(newSelectable.id);
      setSelectedTree(newSelectable.tree);
    },
    undefined,
    [hasFocus, selectableRequests, selectedId, setSelectedId, setSelectedTree],
  );

  useKey(
    'ArrowDown',
    (e) => {
      if (!hasFocus) return;
      e.preventDefault();
      const i = selectableRequests.findIndex((r) => r.id === selectedId);
      const newSelectable = selectableRequests[i + 1];
      if (newSelectable == null) {
        return;
      }

      setSelectedId(newSelectable.id);
      setSelectedTree(newSelectable.tree);
    },
    undefined,
    [hasFocus, selectableRequests, selectedId, setSelectedId, setSelectedTree],
  );

  const handleMove = useCallback<SidebarItemProps['onMove']>(
    (id, side) => {
      let hoveredTree = treeParentMap[id] ?? null;
      const dragIndex = hoveredTree?.children.findIndex((n) => n.item.id === id) ?? -99;
      const hoveredItem = hoveredTree?.children[dragIndex]?.item ?? null;
      let hoveredIndex = dragIndex + (side === 'above' ? 0 : 1);

      if (hoveredItem?.model === 'folder' && side === 'below' && !isCollapsed(hoveredItem.id)) {
        // Move into the folder if it's open and we're moving below it
        hoveredTree = hoveredTree?.children.find((n) => n.item.id === id) ?? null;
        hoveredIndex = 0;
      }

      setHoveredTree(hoveredTree);
      setHoveredIndex(hoveredIndex);
    },
    [isCollapsed, treeParentMap],
  );

  const handleDragStart = useCallback<SidebarItemProps['onDragStart']>((id: string) => {
    setDraggingId(id);
  }, []);

  const handleEnd = useCallback<SidebarItemProps['onEnd']>(
    async (itemId) => {
      setHoveredTree(null);
      handleClearSelected();

      if (hoveredTree == null || hoveredIndex == null) {
        return;
      }

      // Block dragging folder into itself
      if (hoveredTree.item.id === itemId) {
        return;
      }

      const parentTree = treeParentMap[itemId] ?? null;
      const index = parentTree?.children.findIndex((n) => n.item.id === itemId) ?? -1;
      const child = parentTree?.children[index ?? -1];
      if (child == null || parentTree == null) return;

      const movedToDifferentTree = hoveredTree.item.id !== parentTree.item.id;
      const movedUpInSameTree = !movedToDifferentTree && hoveredIndex < index;

      const newChildren = hoveredTree.children.filter((c) => c.item.id !== itemId);
      if (movedToDifferentTree || movedUpInSameTree) {
        // Moving up or into a new tree is simply inserting before the hovered item
        newChildren.splice(hoveredIndex, 0, child);
      } else {
        // Moving down has to account for the fact that the original item will be removed
        newChildren.splice(hoveredIndex - 1, 0, child);
      }

      const insertedIndex = newChildren.findIndex((c) => c.item === child.item);
      const prev = newChildren[insertedIndex - 1]?.item;
      const next = newChildren[insertedIndex + 1]?.item;
      const beforePriority = prev == null || prev.model === 'workspace' ? 0 : prev.sortPriority;
      const afterPriority = next == null || next.model === 'workspace' ? 0 : next.sortPriority;

      const folderId = hoveredTree.item.model === 'folder' ? hoveredTree.item.id : null;
      const shouldUpdateAll = afterPriority - beforePriority < 1;

      if (shouldUpdateAll) {
        await Promise.all(
          newChildren.map((child, i) => {
            const sortPriority = i * 1000;
            if (child.item.model === 'folder') {
              const updateFolder = (f: Folder) => ({ ...f, sortPriority, folderId });
              return updateAnyFolder.mutateAsync({ id: child.item.id, update: updateFolder });
            } else if (child.item.model === 'grpc_request') {
              const updateRequest = (r: GrpcRequest) => ({ ...r, sortPriority, folderId });
              return updateAnyGrpcRequest.mutateAsync({ id: child.item.id, update: updateRequest });
            } else if (child.item.model === 'http_request') {
              const updateRequest = (r: HttpRequest) => ({ ...r, sortPriority, folderId });
              return updateAnyHttpRequest.mutateAsync({ id: child.item.id, update: updateRequest });
            }
          }),
        );
      } else {
        const sortPriority = afterPriority - (afterPriority - beforePriority) / 2;
        if (child.item.model === 'folder') {
          const updateFolder = (f: Folder) => ({ ...f, sortPriority, folderId });
          await updateAnyFolder.mutateAsync({ id: child.item.id, update: updateFolder });
        } else if (child.item.model === 'grpc_request') {
          const updateRequest = (r: GrpcRequest) => ({ ...r, sortPriority, folderId });
          await updateAnyGrpcRequest.mutateAsync({ id: child.item.id, update: updateRequest });
        } else if (child.item.model === 'http_request') {
          const updateRequest = (r: HttpRequest) => ({ ...r, sortPriority, folderId });
          await updateAnyHttpRequest.mutateAsync({ id: child.item.id, update: updateRequest });
        }
      }
      setDraggingId(null);
    },
    [
      handleClearSelected,
      hoveredTree,
      hoveredIndex,
      treeParentMap,
      updateAnyFolder,
      updateAnyGrpcRequest,
      updateAnyHttpRequest,
    ],
  );

  const [showMainContextMenu, setShowMainContextMenu] = useState<{
    x: number;
    y: number;
  } | null>(null);

  const handleMainContextMenu = useCallback((e: React.MouseEvent<HTMLElement>) => {
    e.preventDefault();
    e.stopPropagation();
    setShowMainContextMenu({ x: e.clientX, y: e.clientY });
  }, []);

  const mainContextMenuItems = useCreateDropdownItems();

  // Not ready to render yet
  if (tree == null || collapsed.value == null) {
    return null;
  }

  return (
    <aside
      aria-hidden={hidden}
      ref={sidebarRef}
      onFocus={handleFocus}
      onBlur={handleBlur}
      tabIndex={hidden ? -1 : 0}
      onContextMenu={handleMainContextMenu}
      className={classNames(className, 'h-full grid grid-rows-[minmax(0,1fr)_auto]')}
    >
      <div className="pb-3 overflow-x-visible overflow-y-scroll pt-2">
        <ContextMenu
          triggerPosition={showMainContextMenu}
          items={mainContextMenuItems}
          onClose={() => setShowMainContextMenu(null)}
        />
        <SidebarItems
          treeParentMap={treeParentMap}
          activeId={activeRequest?.id ?? null}
          selectedId={selectedId}
          selectedTree={selectedTree}
          isCollapsed={isCollapsed}
          httpRequestActions={httpRequestActions}
          httpResponses={httpResponses}
          grpcConnections={grpcConnections}
          tree={tree}
          focused={hasFocus}
          draggingId={draggingId}
          onSelect={handleSelect}
          hoveredIndex={hoveredIndex}
          hoveredTree={hoveredTree}
          handleMove={handleMove}
          handleEnd={handleEnd}
          handleDragStart={handleDragStart}
        />
      </div>
      <SyncDropdown />
    </aside>
  );
}

interface SidebarItemsProps {
  tree: TreeNode;
  focused: boolean;
  draggingId: string | null;
  activeId: string | null;
  selectedId: string | null;
  selectedTree: TreeNode | null;
  treeParentMap: Record<string, TreeNode>;
  hoveredTree: TreeNode | null;
  hoveredIndex: number | null;
  handleMove: (id: string, side: 'above' | 'below') => void;
  handleEnd: (id: string) => void;
  handleDragStart: (id: string) => void;
  onSelect: (requestId: string) => void;
  isCollapsed: (id: string) => boolean;
  httpRequestActions: CallableHttpRequestAction[];
  httpResponses: HttpResponse[];
  grpcConnections: GrpcConnection[];
}

function SidebarItems({
  tree,
  focused,
  activeId,
  selectedId,
  selectedTree,
  draggingId,
  onSelect,
  treeParentMap,
  isCollapsed,
  hoveredTree,
  hoveredIndex,
  handleEnd,
  handleMove,
  handleDragStart,
  httpRequestActions,
  httpResponses,
  grpcConnections,
}: SidebarItemsProps) {
  return (
    <VStack
      as="ul"
      role="menu"
      aria-orientation="vertical"
      dir="ltr"
      className={classNames(
        tree.depth > 0 && 'border-l border-border-subtle',
        tree.depth === 0 && 'ml-0',
        tree.depth >= 1 && 'ml-[1.2rem]',
      )}
    >
      {tree.children.map((child, i) => {
        const selected = selectedId === child.item.id;
        const active = activeId === child.item.id;
        return (
          <Fragment key={child.item.id}>
            {hoveredIndex === i && hoveredTree?.item.id === tree.item.id && <DropMarker />}
            <SidebarItem
              selected={selected}
              itemId={child.item.id}
              itemName={child.item.name}
              itemFallbackName={
                child.item.model === 'http_request' || child.item.model === 'grpc_request'
                  ? resolvedModelName(child.item)
                  : 'New Folder'
              }
              itemModel={child.item.model}
              itemPrefix={
                (child.item.model === 'http_request' || child.item.model === 'grpc_request') && (
                  <HttpMethodTag
                    request={child.item}
                    className={classNames(!(active || selected) && 'text-text-subtlest')}
                  />
                )
              }
              httpRequestActions={httpRequestActions}
              latestHttpResponse={httpResponses.find((r) => r.requestId === child.item.id) ?? null}
              latestGrpcConnection={
                grpcConnections.find((c) => c.requestId === child.item.id) ?? null
              }
              onMove={handleMove}
              onEnd={handleEnd}
              onSelect={onSelect}
              onDragStart={handleDragStart}
              useProminentStyles={focused}
              isCollapsed={isCollapsed}
              child={child}
            >
              {child.item.model === 'folder' &&
                !isCollapsed(child.item.id) &&
                draggingId !== child.item.id && (
                  <SidebarItems
                    activeId={activeId}
                    draggingId={draggingId}
                    focused={focused}
                    handleDragStart={handleDragStart}
                    handleEnd={handleEnd}
                    handleMove={handleMove}
                    hoveredIndex={hoveredIndex}
                    hoveredTree={hoveredTree}
                    httpRequestActions={httpRequestActions}
                    httpResponses={httpResponses}
                    grpcConnections={grpcConnections}
                    isCollapsed={isCollapsed}
                    onSelect={onSelect}
                    selectedId={selectedId}
                    selectedTree={selectedTree}
                    tree={child}
                    treeParentMap={treeParentMap}
                  />
                )}
            </SidebarItem>
          </Fragment>
        );
      })}
      {hoveredIndex === tree.children.length && hoveredTree?.item.id === tree.item.id && (
        <DropMarker />
      )}
    </VStack>
  );
}

type SidebarItemProps = {
  className?: string;
  itemId: string;
  itemName: string;
  itemFallbackName: string;
  itemModel: AnyModel['model'];
  itemPrefix: ReactNode;
  useProminentStyles?: boolean;
  selected: boolean;
  onMove: (id: string, side: 'above' | 'below') => void;
  onEnd: (id: string) => void;
  onDragStart: (id: string) => void;
  children?: ReactNode;
  child: TreeNode;
  latestHttpResponse: HttpResponse | null;
  latestGrpcConnection: GrpcConnection | null;
} & Pick<SidebarItemsProps, 'isCollapsed' | 'onSelect' | 'httpRequestActions'>;

type DragItem = {
  id: string;
  itemName: string;
};

function SidebarItem({
  itemName,
  itemId,
  itemModel,
  child,
  onMove,
  onEnd,
  onDragStart,
  onSelect,
  isCollapsed,
  itemPrefix,
  className,
  selected,
  itemFallbackName,
  useProminentStyles,
  latestHttpResponse,
  latestGrpcConnection,
  httpRequestActions,
  children,
}: SidebarItemProps) {
  const ref = useRef<HTMLLIElement>(null);

  const [, connectDrop] = useDrop<DragItem, void>(
    {
      accept: ItemTypes.REQUEST,
      hover: (_, monitor) => {
        if (!ref.current) return;
        const hoverBoundingRect = ref.current?.getBoundingClientRect();
        const hoverMiddleY = (hoverBoundingRect.bottom - hoverBoundingRect.top) / 2;
        const clientOffset = monitor.getClientOffset();
        const hoverClientY = (clientOffset as XYCoord).y - hoverBoundingRect.top;
        onMove(itemId, hoverClientY < hoverMiddleY ? 'above' : 'below');
      },
    },
    [onMove],
  );

  const [, connectDrag] = useDrag<
    DragItem,
    unknown,
    {
      isDragging: boolean;
    }
  >(
    () => ({
      type: ItemTypes.REQUEST,
      item: () => {
        onDragStart(itemId);
        return { id: itemId, itemName };
      },
      collect: (m) => ({ isDragging: m.isDragging() }),
      options: { dropEffect: 'move' },
      end: () => onEnd(itemId),
    }),
    [onEnd],
  );

  connectDrag(connectDrop(ref));

  const activeRequest = useActiveRequest();
  const deleteFolder = useDeleteFolder(itemId);
  const deleteRequest = useDeleteRequest(itemId);
  const renameRequest = useRenameRequest(itemId);
  const duplicateHttpRequest = useDuplicateHttpRequest({ id: itemId, navigateAfter: true });
  const duplicateGrpcRequest = useDuplicateGrpcRequest({ id: itemId, navigateAfter: true });
  const sendRequest = useSendAnyHttpRequest();
  const moveToWorkspace = useMoveToWorkspace(itemId);
  const sendManyRequests = useSendManyRequests();
  const updateHttpRequest = useUpdateAnyHttpRequest();
  const workspaces = useWorkspaces();
  const updateGrpcRequest = useUpdateAnyGrpcRequest();
  const updateAnyFolder = useUpdateAnyFolder();
  const prompt = usePrompt();
  const [editing, setEditing] = useState<boolean>(false);
  const isActive = activeRequest?.id === itemId;
  const createDropdownItems = useCreateDropdownItems({ folderId: itemId });

  useScrollIntoView(ref.current, isActive);

  const handleSubmitNameEdit = useCallback(
    async (el: HTMLInputElement) => {
      if (itemModel === 'http_request') {
        await updateHttpRequest.mutateAsync({
          id: itemId,
          update: (r) => ({ ...r, name: el.value }),
        });
      } else if (itemModel === 'grpc_request') {
        await updateGrpcRequest.mutateAsync({
          id: itemId,
          update: (r) => ({ ...r, name: el.value }),
        });
      }
      setEditing(false);
    },
    [itemId, itemModel, updateGrpcRequest, updateHttpRequest],
  );

  const handleFocus = useCallback((el: HTMLInputElement | null) => {
    el?.focus();
    el?.select();
  }, []);

  const handleInputKeyDown = useCallback(
    async (e: React.KeyboardEvent<HTMLInputElement>) => {
      e.stopPropagation();
      switch (e.key) {
        case 'Enter':
          e.preventDefault();
          await handleSubmitNameEdit(e.currentTarget);
          break;
        case 'Escape':
          e.preventDefault();
          setEditing(false);
          break;
      }
    },
    [handleSubmitNameEdit],
  );

  const handleStartEditing = useCallback(() => {
    if (itemModel !== 'http_request' && itemModel !== 'grpc_request') return;
    setEditing(true);
  }, [setEditing, itemModel]);

  const handleBlur = useCallback(
    async (e: React.FocusEvent<HTMLInputElement>) => {
      await handleSubmitNameEdit(e.currentTarget);
    },
    [handleSubmitNameEdit],
  );

  const handleSelect = useCallback(() => onSelect(itemId), [onSelect, itemId]);
  const [showContextMenu, setShowContextMenu] = useState<{
    x: number;
    y: number;
  } | null>(null);

  const handleCloseContextMenu = useCallback(() => {
    setShowContextMenu(null);
  }, []);

  const handleContextMenu = useCallback((e: React.MouseEvent<HTMLElement>) => {
    e.preventDefault();
    e.stopPropagation();
    setShowContextMenu({ x: e.clientX, y: e.clientY });
  }, []);

  const items = useMemo<DropdownItem[]>(() => {
    if (itemModel === 'folder') {
      return [
        {
          key: 'sendAll',
          label: 'Send All',
          leftSlot: <Icon icon="send_horizontal" />,
          onSelect: () => sendManyRequests.mutate(child.children.map((c) => c.item.id)),
        },
        {
          key: 'rename',
          label: 'Rename',
          leftSlot: <Icon icon="pencil" />,
          onSelect: async () => {
            const name = await prompt({
              id: 'rename-folder',
              title: 'Rename Folder',
              description: (
                <>
                  Enter a new name for <InlineCode>{itemName}</InlineCode>
                </>
              ),
              confirmText: 'Save',
              label: 'Name',
              placeholder: 'New Name',
              defaultValue: itemName,
            });
            if (name == null) return;
            updateAnyFolder.mutate({ id: itemId, update: (f) => ({ ...f, name }) });
          },
        },
        {
          key: 'deleteFolder',
          label: 'Delete',
          variant: 'danger',
          leftSlot: <Icon icon="trash" />,
          onSelect: () => deleteFolder.mutate(),
        },
        { type: 'separator' },
        ...createDropdownItems,
      ];
    } else {
      const requestItems: DropdownItem[] =
        itemModel === 'http_request'
          ? [
              {
                key: 'sendRequest',
                label: 'Send',
                hotKeyAction: 'http_request.send',
                hotKeyLabelOnly: true, // Already bound in URL bar
                leftSlot: <Icon icon="send_horizontal" />,
                onSelect: () => sendRequest.mutate(itemId),
              },
              ...httpRequestActions.map((a) => ({
                key: a.key,
                label: a.label,
                // eslint-disable-next-line @typescript-eslint/no-explicit-any
                leftSlot: <Icon icon={(a.icon as any) ?? 'empty'} />,
                onSelect: async () => {
                  const request = await getHttpRequest(itemId);
                  if (request != null) await a.call(request);
                },
              })),
              { type: 'separator' },
            ]
          : [];
      return [
        ...requestItems,
        {
          key: 'renameRequest',
          label: 'Rename',
          leftSlot: <Icon icon="pencil" />,
          onSelect: renameRequest.mutate,
        },
        {
          key: 'duplicateRequest',
          label: 'Duplicate',
          hotKeyAction: 'http_request.duplicate',
          hotKeyLabelOnly: true, // Would trigger for every request (bad)
          leftSlot: <Icon icon="copy" />,
          onSelect: () =>
            itemModel === 'http_request'
              ? duplicateHttpRequest.mutate()
              : duplicateGrpcRequest.mutate(),
        },
        {
          key: 'moveWorkspace',
          label: 'Move',
          leftSlot: <Icon icon="arrow_right_circle" />,
          hidden: workspaces.length <= 1,
          onSelect: moveToWorkspace.mutate,
        },
        {
          key: 'deleteRequest',
          variant: 'danger',
          label: 'Delete',
          leftSlot: <Icon icon="trash" />,
          onSelect: () => deleteRequest.mutate(),
        },
      ];
    }
  }, [
    child.children,
    createDropdownItems,
    deleteFolder,
    deleteRequest,
    duplicateGrpcRequest,
    duplicateHttpRequest,
    httpRequestActions,
    itemId,
    itemModel,
    itemName,
    moveToWorkspace.mutate,
    prompt,
    renameRequest.mutate,
    sendManyRequests,
    sendRequest,
    updateAnyFolder,
    workspaces.length,
  ]);

  return (
    <li ref={ref} draggable>
      <div className={classNames(className, 'block relative group/item px-1.5 pb-0.5')}>
        <ContextMenu
          triggerPosition={showContextMenu}
          items={items}
          onClose={handleCloseContextMenu}
        />
        <button
          // tabIndex={-1} // Will prevent drag-n-drop
          disabled={editing}
          onClick={handleSelect}
          onDoubleClick={handleStartEditing}
          onContextMenu={handleContextMenu}
          data-active={isActive}
          data-selected={selected}
          className={classNames(
            'w-full flex gap-1.5 items-center h-xs px-1.5 rounded-md focus-visible:ring focus-visible:ring-border-focus outline-0',
            editing && 'ring-1 focus-within:ring-focus',
            isActive && 'bg-surface-highlight text-text',
            !isActive && 'text-text-subtle group-hover/item:text-text',
            showContextMenu && '!text-text', // Show as "active" when context menu is open
            selected && useProminentStyles && '!bg-surface-active',
          )}
        >
          {itemModel === 'folder' && (
            <Icon
              size="sm"
              icon="chevron_right"
              className={classNames(
                'text-text-subtlest',
                'transition-transform',
                !isCollapsed(itemId) && 'transform rotate-90',
              )}
            />
          )}
          <div className="flex items-center gap-2 min-w-0">
            {itemPrefix}
            {editing ? (
              <input
                ref={handleFocus}
                defaultValue={itemName}
                className="bg-transparent outline-none w-full"
                onBlur={handleBlur}
                onKeyDown={handleInputKeyDown}
              />
            ) : (
              <span className="truncate">{itemName || itemFallbackName}</span>
            )}
          </div>
          {latestGrpcConnection ? (
            <div className="ml-auto">
              {isResponseLoading(latestGrpcConnection) && (
                <Icon spin size="sm" icon="update" className="text-text-subtlest" />
              )}
            </div>
          ) : latestHttpResponse ? (
            <div className="ml-auto">
              {isResponseLoading(latestHttpResponse) ? (
                <Icon spin size="sm" icon="refresh" className="text-text-subtlest" />
              ) : (
                <StatusTag className="text-xs" response={latestHttpResponse} />
              )}
            </div>
          ) : null}
        </button>
      </div>
      {children}
    </li>
  );
}
