import { createStore } from "solid-js/store";

export type SortField = "modified_at" | "added_at" | "created_at" | "filename" | "format" | "size" | "rating";
export type SortOrder = "asc" | "desc";
export type ViewLayout = "masonry-v" | "masonry-h" | "grid" | "list";

interface FilterState {
  selectedTags: number[];
  selectedFolderId: number | null;
  folderRecursiveView: boolean;
  filterUntagged: boolean;
  searchQuery: string;
  sortBy: SortField;
  sortOrder: SortOrder;
  layout: ViewLayout;
  thumbSize: number;
}

const STORAGE_KEY = "elleven-filter-preference";

const defaultState: FilterState = {
  selectedTags: [],
  selectedFolderId: null,
  folderRecursiveView: false,
  filterUntagged: false,
  searchQuery: "",
  sortBy: "modified_at",
  sortOrder: "desc",
  layout: "masonry-v",
  thumbSize: 200
};

const getPersisted = (): Partial<FilterState> => {
    try {
        const saved = localStorage.getItem(STORAGE_KEY);
        return saved ? JSON.parse(saved) : {};
    } catch {
        return {};
    }
};

const persisted = getPersisted();

const [filterState, setFilterState] = createStore<FilterState>({
  ...defaultState,
  ...persisted,
  // Don't persist these
  selectedTags: [],
  selectedFolderId: null,
  filterUntagged: false,
  searchQuery: ""
});

const persist = (newState: Partial<FilterState>) => {
    const toSave = {
        sortBy: filterState.sortBy,
        sortOrder: filterState.sortOrder,
        layout: filterState.layout,
        thumbSize: filterState.thumbSize,
        ...newState
    };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(toSave));
};

export const filterActions = {
  toggleTag: (tagId: number) => {
    const current = filterState.selectedTags;
    if (current.includes(tagId)) {
      setFilterState("selectedTags", (tags) => tags.filter(id => id !== tagId));
    } else {
      setFilterState({
        selectedTags: [...current, tagId],
        filterUntagged: false 
      });
    }
  },

  setUntagged: (isActive: boolean) => {
    setFilterState("filterUntagged", isActive);
    if (isActive) {
      setFilterState("selectedTags", []);
    }
  },

  toggleUntagged: () => {
    filterActions.setUntagged(!filterState.filterUntagged);
  },

  setFolder: (folderId: number | null) => {
    setFilterState("selectedFolderId", folderId);
  },

  setFolderRecursiveView: (isRecursive: boolean) => {
    setFilterState("folderRecursiveView", isRecursive);
  },

  setSearch: (query: string) => {
    setFilterState("searchQuery", query);
  },

  setSortBy: (field: SortField) => {
    setFilterState("sortBy", field);
    persist({ sortBy: field });
  },

  setSortOrder: (order: SortOrder) => {
    setFilterState("sortOrder", order);
    persist({ sortOrder: order });
  },

  setLayout: (layout: ViewLayout) => {
    setFilterState("layout", layout);
    persist({ layout: layout });
  },

  setThumbSize: (size: number) => {
    setFilterState("thumbSize", size);
    persist({ thumbSize: size });
  },

  clearAll: () => {
    setFilterState({
      ...defaultState,
      // Keep persistent ones but let user clear search/tags
      selectedTags: [],
      selectedFolderId: null,
      filterUntagged: false,
      searchQuery: ""
    });
  },

  hasActiveFilters: () => {
    return filterState.selectedTags.length > 0 || 
           filterState.filterUntagged || 
           filterState.selectedFolderId !== null;
  }
};

export { filterState };
