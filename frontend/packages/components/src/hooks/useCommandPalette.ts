import * as React from "react";
import { type LucideIcon } from "lucide-react";
export interface CommandItem {
  id: string;
  title: string;
  subtitle?: string;
  icon?: LucideIcon;
  shortcut?: string;
  category: "navigation" | "asset" | "action" | "recent";
  metadata?: Record<string, unknown>;
  onSelect: () => void;
}

interface UseCommandPaletteOptions {
  items: CommandItem[];
  onOpen?: () => void;
  onClose?: () => void;
}

interface UseCommandPaletteReturn {
  isOpen: boolean;
  searchQuery: string;
  setSearchQuery: (query: string) => void;
  selectedIndex: number;
  setSelectedIndex: (index: number) => void;
  filteredItems: CommandItem[];
  groupedItems: GroupedItems;
  recentItems: CommandItem[];
  open: () => void;
  close: () => void;
  toggle: () => void;
}

interface GroupedItems {
  navigation: CommandItem[];
  asset: CommandItem[];
  action: CommandItem[];
  recent: CommandItem[];
}

// Note: Recent searches functionality is defined but not currently used
// const RECENT_SEARCHES_KEY = "command-palette-recent-searches";
// const MAX_RECENT_SEARCHES = 5;

// function getRecentSearches(): CommandItem[] {
//   if (typeof window === "undefined") return [];
//   try {
//     const stored = localStorage.getItem(RECENT_SEARCHES_KEY);
//     if (stored) {
//       return JSON.parse(stored);
//     }
//   } catch {
//     // Ignore localStorage errors
//   }
//   return [];
// }

// Note: Recent searches functionality - getRecentSearches is used but simplified
function getRecentSearches(): CommandItem[] {
  if (typeof window === "undefined") return [];
  try {
    const stored = localStorage.getItem("command-palette-recent-searches");
    if (stored) {
      return JSON.parse(stored);
    }
  } catch {
    // Ignore localStorage errors
  }
  return [];
}

function filterItems(items: CommandItem[], query: string): CommandItem[] {
  if (!query.trim()) {
    return items;
  }
  const lowerQuery = query.toLowerCase();
  return items.filter(
    (item) =>
      item.title.toLowerCase().includes(lowerQuery) ||
      item.subtitle?.toLowerCase().includes(lowerQuery),
  );
}

function groupItems(items: CommandItem[]): GroupedItems {
  const groups: GroupedItems = {
    navigation: [],
    asset: [],
    action: [],
    recent: [],
  };

  for (const item of items) {
    if (groups[item.category]) {
      groups[item.category].push(item);
    }
  }

  return groups;
}

export function useCommandPalette({
  items,
  onOpen,
  onClose,
}: UseCommandPaletteOptions): UseCommandPaletteReturn {
  const [isOpen, setIsOpen] = React.useState(false);
  const [searchQuery, setSearchQuery] = React.useState("");
  const [selectedIndex, setSelectedIndex] = React.useState(0);
  const [recentItems, setRecentItems] = React.useState<CommandItem[]>([]);

  const filteredItems = React.useMemo(
    () => filterItems(items, searchQuery),
    [items, searchQuery],
  );

  const groupedItems = React.useMemo(
    () => groupItems(filteredItems),
    [filteredItems],
  );

  React.useEffect(() => {
    setRecentItems(getRecentSearches());
  }, []);

  React.useEffect(() => {
    setSelectedIndex(0);
  }, [searchQuery]);

  const open = React.useCallback(() => {
    setIsOpen(true);
    setRecentItems(getRecentSearches());
    onOpen?.();
  }, [onOpen]);

  const close = React.useCallback(() => {
    setIsOpen(false);
    setSearchQuery("");
    setSelectedIndex(0);
    onClose?.();
  }, [onClose]);

  const toggle = React.useCallback(() => {
    if (isOpen) {
      close();
    } else {
      open();
    }
  }, [isOpen, open, close]);

  React.useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        toggle();
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [toggle]);

  return {
    isOpen,
    searchQuery,
    setSearchQuery,
    selectedIndex,
    setSelectedIndex,
    filteredItems,
    groupedItems,
    recentItems,
    open,
    close,
    toggle,
  };
}

export type { GroupedItems };
