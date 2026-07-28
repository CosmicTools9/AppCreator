/**
 * InlineEditTable - 行内编辑表格组件
 *
 * 类似飞书多维表格的交互体验：
 * - 点击单元格进入编辑模式
 * - Enter / Blur 提交变更
 * - Escape 取消编辑
 */

import * as React from "react";
import {
  type SortingState,
  type VisibilityState,
  type PaginationState,
  flexRender,
  getCoreRowModel,
  getSortedRowModel,
  getPaginationRowModel,
  useReactTable,
} from "@tanstack/react-table";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../ui/table";
import { Skeleton } from "../ui/skeleton";
import { ChevronDown, ChevronUp, ChevronsUpDown, Trash2 } from "lucide-react";
import { cn } from "../../lib/utils";
import { useT } from "@alioth/i18n";
import { DataTablePagination } from "./DataTablePagination";

export interface InlineEditColumnDef<TData> {
  id?: string;
  accessorKey?: string;
  accessorFn?: (originalRow: TData) => unknown;
  header?: React.ReactNode | ((props: { column: { id: string } }) => React.ReactNode);
  cell?: React.ReactNode | ((props: { row: { original: TData }; getValue: () => unknown }) => React.ReactNode);
  enableSorting?: boolean;
  editable?: boolean;
}

export interface InlineEditTableProps<TData = any> {
  columns: InlineEditColumnDef<TData>[];
  data: TData[];
  isLoading?: boolean;
  enableSorting?: boolean;
  enablePagination?: boolean;
  pageSize?: number;
  pageSizeOptions?: number[];
  emptyMessage?: string;
  className?: string;
  pageCount?: number;
  pageIndex?: number;
  totalCount?: number;
  onPageChange?: (pageIndex: number) => void;
  onPageSizeChange?: (pageSize: number) => void;

  // 行内编辑
  editingRowId?: string | number | null;
  editingColumnId?: string | null;
  editValues?: Record<string, string>;
  onCellClick?: (row: TData, columnId: string) => void;
  onCellChange?: (columnId: string, value: string) => void;
  onCellCommit?: (row: TData, columnId?: string, value?: string) => void;
  onCellCancel?: (row?: TData, columnId?: string) => void;
  onRowDelete?: (row: TData) => void;
  // 行内新增（与 SmartCrudTable 一致的模式）
  isNewRowEditing?: boolean;
  newRowValues?: Record<string, string>;
  onNewRowCellChange?: (columnId: string, value: string) => void;
  onNewRowCommit?: () => void;
  onNewRowCancel?: () => void;
}

export function InlineEditTable<TData = any>({
  columns,
  data,
  isLoading = false,
  enableSorting = true,
  enablePagination = true,
  pageSize = 10,
  pageSizeOptions = [10, 20, 50, 100],
  emptyMessage,
  className,
  pageCount: controlledPageCount,
  pageIndex: controlledPageIndex,
  totalCount,
  onPageChange,
  onPageSizeChange,

  // 编辑
  editingRowId,
  editingColumnId,
  editValues = {},
  onCellClick,
  onCellChange,
  onCellCommit,
  onCellCancel,
  onRowDelete,
  // 行内新增
  isNewRowEditing,
  newRowValues = {},
  onNewRowCellChange,
  onNewRowCommit,
  onNewRowCancel,
}: InlineEditTableProps<TData>) {
  const t = useT();
  const commitLockRef = React.useRef<Set<string>>(new Set());
  const getCommitKey = (rId: string | number, cId: string) => `${rId}::${cId}`;
  const resolvedEmptyMessage = emptyMessage ?? t("common.empty");
  const isControlledPagination = controlledPageCount !== undefined;

  const [sorting, setSorting] = React.useState<SortingState>([]);
  const [columnVisibility] = React.useState<VisibilityState>({});
  const [pagination, setPagination] = React.useState<PaginationState>({
    pageIndex: controlledPageIndex ?? 0,
    pageSize: enablePagination ? pageSize : data.length,
  });

  React.useEffect(() => {
    if (controlledPageIndex !== undefined) {
      setPagination((prev) => ({ ...prev, pageIndex: controlledPageIndex }));
    }
  }, [controlledPageIndex]);

  const handlePaginationChange: typeof setPagination = (updater) => {
    setPagination(updater);
    if (onPageChange || onPageSizeChange) {
      const next = typeof updater === "function" ? updater(pagination) : updater;
      if (next.pageIndex !== pagination.pageIndex) {
        onPageChange?.(next.pageIndex);
      }
      if (next.pageSize !== pagination.pageSize) {
        onPageSizeChange?.(next.pageSize);
      }
    }
  };

  const table = useReactTable({
    data,
    columns: columns as any,
    getCoreRowModel: getCoreRowModel(),
    onSortingChange: setSorting,
    getSortedRowModel: getSortedRowModel(),
    getPaginationRowModel:
      enablePagination && !isControlledPagination
        ? getPaginationRowModel()
        : undefined,
    onPaginationChange: enablePagination ? handlePaginationChange : undefined,
    manualPagination: isControlledPagination,
    pageCount: isControlledPagination ? controlledPageCount : undefined,
    state: {
      sorting,
      columnVisibility,
      pagination: enablePagination ? pagination : undefined,
    },
  });

  const renderSortIcon = (sorted: false | "asc" | "desc") => {
    if (!sorted) {
      return <ChevronsUpDown className="h-4 w-4 text-muted-foreground" />;
    }
    return sorted === "asc" ? (
      <ChevronUp className="h-4 w-4 text-primary" />
    ) : (
      <ChevronDown className="h-4 w-4 text-primary" />
    );
  };

  const getRowId = (row: TData): string | number => {
    return ((row as any).id as string | number) ?? "";
  };

  if (isLoading) {
    return (
      <div className={cn("rounded-md border flex flex-col", className)}>
        <div className="flex-1 overflow-auto">
          <Table wrapperClassName="overflow-visible">
            <TableHeader className="sticky top-0 bg-background z-10">
              <TableRow>
                {columns.map((_, index) => (
                  <TableHead key={index}>
                    <Skeleton className="h-4 w-full" />
                  </TableHead>
                ))}
              </TableRow>
            </TableHeader>
            <TableBody>
              {Array.from({ length: pageSize }).map((_, rowIndex) => (
                <TableRow key={rowIndex}>
                  {columns.map((_, colIndex) => (
                    <TableCell key={colIndex}>
                      <Skeleton className="h-4 w-full" />
                    </TableCell>
                  ))}
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </div>
    );
  }

  if (!data || data.length === 0) {
    if (isNewRowEditing) {
      return (
        <div className={cn("rounded-md border flex flex-col overflow-hidden", className)}>
          <div className="flex-1 overflow-auto">
            <Table wrapperClassName="overflow-visible">
              <TableHeader className="sticky top-0 bg-background z-10">
                {table.getHeaderGroups().map((headerGroup) => (
                  <TableRow key={headerGroup.id}>
                    {headerGroup.headers.map((header) => (
                      <TableHead key={header.id}>
                        <div className="flex items-center gap-2">
                          {header.isPlaceholder
                            ? null
                            : flexRender(
                                header.column.columnDef.header,
                                header.getContext(),
                              )}
                        </div>
                      </TableHead>
                    ))}
                    {onRowDelete && <TableHead className="w-16">{t("meta.common.actions")}</TableHead>}
                  </TableRow>
                ))}
              </TableHeader>
              <TableBody>
                {isNewRowEditing && (
                  <TableRow
                    data-state="new-row"
                    className="bg-primary/[0.04] border-b-2 border-primary/20"
                  >
                    {columns.map((col) => {
                      const colId = col.id ?? col.accessorKey ?? "";
                      const isEditable = col.editable === true;
                      const isFirst = col === columns[0];

                      if (isEditable) {
                        return (
                          <TableCell key={colId} className="p-0">
                            <input
                              type="text"
                              autoFocus={isFirst}
                              value={newRowValues[colId] ?? ""}
                              placeholder={typeof col.header === "string" ? col.header : ""}
                              onChange={(e) => onNewRowCellChange?.(colId, e.target.value)}
                              onKeyDown={(e) => {
                                if (e.key === "Enter") {
                                  e.preventDefault();
                                  onNewRowCommit?.();
                                }
                                if (e.key === "Escape") {
                                  e.preventDefault();
                                  onNewRowCancel?.();
                                }
                              }}
                              className="w-full h-full px-4 py-3 bg-background text-foreground border-2 border-dashed border-primary/50 rounded-none focus:outline-none focus:ring-0 focus:border-primary"
                            />
                          </TableCell>
                        );
                      }

                      return (
                        <TableCell key={colId}>
                          <span className="text-muted-foreground">—</span>
                        </TableCell>
                      );
                    })}
                    {onRowDelete && (
                      <TableCell>
                        <span className="text-muted-foreground">—</span>
                      </TableCell>
                    )}
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </div>
        </div>
      );
    }
    return (
      <div
        className={cn(
          "flex items-center justify-center rounded-md border py-12",
          className,
        )}
      >
        <p className="text-sm text-muted-foreground">{resolvedEmptyMessage}</p>
      </div>
    );
  }

  return (
    <div className={cn("rounded-md border flex flex-col overflow-hidden", className)}>
      <div className="flex-1 overflow-auto">
        <Table wrapperClassName="overflow-visible">
          <TableHeader className="sticky top-0 bg-background z-10">
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map((header) => (
                  <TableHead
                    key={header.id}
                    className={cn(
                      enableSorting &&
                        header.column.getCanSort() &&
                        "cursor-pointer select-none",
                    )}
                  >
                    <div className="flex items-center gap-2">
                      {header.isPlaceholder
                        ? null
                        : flexRender(
                            header.column.columnDef.header,
                            header.getContext(),
                          )}
                      {enableSorting && header.column.getCanSort() && (
                        <button
                          onClick={header.column.getToggleSortingHandler()}
                          className="p-0"
                        >
                          {renderSortIcon(
                            header.column.getIsSorted() as false | "asc" | "desc",
                          )}
                        </button>
                      )}
                    </div>
                  </TableHead>
                ))}
                {onRowDelete && <TableHead className="w-16">{t("meta.common.actions")}</TableHead>}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {isNewRowEditing && (
              <TableRow
                data-state="new-row"
                className="bg-primary/[0.04] border-b-2 border-primary/20"
              >
                {columns.map((col) => {
                  const colId = col.id ?? col.accessorKey ?? "";
                  const isEditable = col.editable === true;
                  const isFirst = col === columns[0];

                  if (isEditable) {
                    return (
                      <TableCell key={colId} className="p-0">
                        <input
                          type="text"
                          autoFocus={isFirst}
                          value={newRowValues[colId] ?? ""}
                          placeholder={typeof col.header === "string" ? col.header : ""}
                          onChange={(e) => onNewRowCellChange?.(colId, e.target.value)}
                          onKeyDown={(e) => {
                            if (e.key === "Enter") {
                              e.preventDefault();
                              onNewRowCommit?.();
                            }
                            if (e.key === "Escape") {
                              e.preventDefault();
                              onNewRowCancel?.();
                            }
                          }}
                          className="w-full h-full px-4 py-3 bg-background text-foreground border-2 border-dashed border-primary/50 rounded-none focus:outline-none focus:ring-0 focus:border-primary"
                        />
                      </TableCell>
                    );
                  }

                  return (
                    <TableCell key={colId}>
                      <span className="text-muted-foreground">—</span>
                    </TableCell>
                  );
                })}
                {onRowDelete && (
                  <TableCell>
                    <span className="text-muted-foreground">—</span>
                  </TableCell>
                )}
              </TableRow>
            )}
            {table.getRowModel().rows.map((row) => {
              const rowId = getRowId(row.original);
              const isEditingRow = editingRowId !== null && editingRowId !== undefined && rowId === editingRowId;

              return (
                <TableRow
                  key={row.id}
                  data-state={isEditingRow ? "editing" : undefined}
                  className={cn(isEditingRow && "bg-primary/5")}
                >
                  {row.getVisibleCells().map((cell) => {
                    const columnDef = cell.column.columnDef as InlineEditColumnDef<TData>;
                    const isEditable = columnDef.editable === true;
                    const colId = cell.column.id;

                    const isEditingCell =
                      editingColumnId !== undefined && editingColumnId !== null
                        ? isEditingRow && isEditable && colId === editingColumnId
                        : isEditingRow && isEditable;

                    if (isEditingCell) {
                      return (
                        <TableCell key={cell.id} className="p-0">
                          <input
                            type="text"
                            autoFocus
                            value={editValues[colId] ?? ""}
                            onChange={(e) => onCellChange?.(colId, e.target.value)}
                            onBlur={() => {
                              onCellCommit?.(row.original, colId, editValues[colId] ?? "");
                            }}
                            onKeyDown={(e) => {
                              if (e.key === "Enter") {
                                e.preventDefault();
                                onCellCommit?.(row.original, colId, editValues[colId] ?? "");
                              }
                              if (e.key === "Escape") {
                                e.preventDefault();
                                onCellCancel?.(row.original, colId);
                              }
                            }}
                            className="w-full h-full px-4 py-3 bg-background text-foreground border-2 border-primary rounded-none focus:outline-none focus:ring-0"
                          />
                        </TableCell>
                      );
                    }

                    return (
                      <TableCell
                        key={cell.id}
                        onClick={() => {
                          if (isEditable && !isEditingRow) {
                            commitLockRef.current.delete(getCommitKey(rowId, colId));
                            onCellClick?.(row.original, colId);
                          }
                        }}
                        className={cn(
                          isEditable && !isEditingRow && "cursor-pointer hover:bg-muted/30",
                        )}
                      >
                        {flexRender(cell.column.columnDef.cell, cell.getContext())}
                      </TableCell>
                    );
                  })}

                  {/* 操作列 */}
                  {onRowDelete && (
                    <TableCell>
                      <div className="flex items-center gap-1">
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            onRowDelete?.(row.original);
                          }}
                          className="h-8 w-8 p-0 inline-flex items-center justify-center rounded-md text-destructive hover:text-destructive/80 hover:bg-destructive/10"
                          title={t("meta.common.delete")}
                        >
                          <Trash2 className="h-4 w-4" />
                        </button>
                      </div>
                    </TableCell>
                  )}
                </TableRow>
              );
            })}
          </TableBody>
        </Table>
      </div>
      {enablePagination && data.length > 0 && (
        <DataTablePagination
          page={table.getState().pagination.pageIndex + 1}
          pageSize={table.getState().pagination.pageSize}
          pageCount={table.getPageCount()}
          totalCount={totalCount ?? data.length}
          pageSizeOptions={pageSizeOptions}
          onPageChange={(p) => table.setPageIndex(p - 1)}
          onPageSizeChange={(size) => table.setPageSize(size)}
        />
      )}
    </div>
  );
}
