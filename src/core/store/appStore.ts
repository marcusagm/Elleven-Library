import { createStore, reconcile } from "solid-js/store";
import { createSignal } from "solid-js";
import { getImages, getLocations, initDb, addLocation } from "../../lib/db";
import { tauriService } from "../tauri/services";
import { listen } from "@tauri-apps/api/event";

export interface ImageItem {
  id: number;
  path: string;
  filename: string;
  width: number | null;
  height: number | null;
  thumbnail_path: string | null;
}

export interface ProgressPayload {
  total: number;
  processed: number;
  current_file: string;
}

// Global Store State
interface AppState {
  items: ImageItem[];
  locations: { path: string; name: string }[];
  selection: number[]; // ID of selected items
}

// Reactive primitives
const [state, setState] = createStore<AppState>({ 
  items: [], 
  locations: [], 
  selection: [] 
});
const [rootPath, setRootPath] = createSignal<string | null>(null);
const [loading, setLoading] = createSignal(true);
const [progress, setProgress] = createSignal<ProgressPayload | null>(null);
const [searchQuery, setSearchQuery] = createSignal("");

// Pagination State (Module Level or inside Store if reactive needed)
let currentOffset = 0;
let isFetching = false;
const BATCH_SIZE = 100;

// Actions
export const appActions = {
  initialize: async () => {
    // ... (keep logic) ...
    try {
      setLoading(true);
      await initDb();
      const locations = await getLocations();
      setState("locations", locations);
      
      if (locations.length > 0) {
        const path = locations[0].path;
        setRootPath(path);
        
        // Start indexing in background
        tauriService.startIndexing({ path }).catch(console.error);
      }

      // Initial Load
      await appActions.refreshImages(true);
      
      // Setup listeners
      appActions.setupListeners();
    } catch (err) {
      console.error("Initialization failed:", err);
    } finally {
      setLoading(false);
    }
  },

  // Reset=true implies a full reload (e.g. after indexing finishes or strict refresh)
  refreshImages: async (reset = false) => {
    if (reset) {
        currentOffset = 0;
        const firstBatch = await getImages(BATCH_SIZE, 0);
        setState("items", reconcile(firstBatch, { key: "id" }));
        currentOffset = BATCH_SIZE;
    } else {
        // Just reload current view? Usually we use loadMore for pagination.
        // For compatibility with listeners, we might want to just reload the visible range
        // But for simplicity, let's reset on full refresh calls.
        currentOffset = 0;
        const fresh = await getImages(BATCH_SIZE, 0);
        setState("items", reconcile(fresh, { key: "id" }));
        currentOffset = BATCH_SIZE;
    }
  },

  loadMore: async () => {
    if (isFetching) return;
    isFetching = true;

    try {
        const nextBatch = await getImages(BATCH_SIZE, currentOffset);
        if (nextBatch.length > 0) {
            setState("items", (prev) => [...prev, ...nextBatch]);
            currentOffset += BATCH_SIZE;
        }
    } finally {
        isFetching = false;
    }
  },

  setRootLocation: async (path: string, name: string) => {
    await addLocation(path, name);
    setState("locations", (locs) => [...locs, { path, name }]);
    setRootPath(path);
    await tauriService.startIndexing({ path });
  },

  toggleSelection: (id: number, multi: boolean) => {
    if (multi) {
      const current = state.selection;
      if (current.includes(id)) {
        setState("selection", current.filter(i => i !== id));
      } else {
        setState("selection", [...current, id]);
      }
    } else {
      setState("selection", [id]);
    }
  },

  setSearch: (query: string) => {
    setSearchQuery(query);
  },
  setupListeners: async () => {
    // Indexer Progress
    await listen<ProgressPayload>("indexer:progress", (event) => {
      setProgress(event.payload);
      // Partial refresh every 10 items to show progress live
      if (event.payload.processed % 10 === 0 || event.payload.processed === event.payload.total) {
        appActions.refreshImages();
      }
    });

    // Indexer Complete
    await listen<number>("indexer:complete", (event) => {
      console.log("Indexer complete. Total:", event.payload);
      setProgress(null);
      // reset=true to ensure we see everything properly sorted
      appActions.refreshImages(true);
    });

    // Thumbnail Generation Updates
    await listen<{ id: number; path: string }>("thumbnail:ready", (event) => {
        setState(
            "items",
            (item) => item.id === event.payload.id,
            "thumbnail_path",
            event.payload.path
        );
    });
  }
};




// Exports for consumption
export const useAppStore = () => {
  return {
    state, // Read-only store (mostly)
    rootPath,
    loading,
    progress,
    searchQuery
  };
};
