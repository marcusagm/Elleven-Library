import { createStore } from "solid-js/store";
import { Tag, tagService } from "../../lib/tags";
import { getLocations } from "../../lib/db";

interface MetadataState {
  tags: Tag[];
  locations: { id: number; path: string; name: string }[];
  libraryStats: {
    total_images: number;
    untagged_images: number;
    tag_counts: Map<number, number>;
    folder_counts: Map<number, number>;
  };
  tagUpdateVersion: number;
}

const [metadataState, setMetadataState] = createStore<MetadataState>({
  tags: [],
  locations: [],
  libraryStats: {
    total_images: 0,
    untagged_images: 0,
    tag_counts: new Map(),
    folder_counts: new Map(),
  },
  tagUpdateVersion: 0
});

export const metadataActions = {
  notifyTagUpdate: () => {
    setMetadataState("tagUpdateVersion", v => v + 1);
    metadataActions.loadStats();
    
    // Check if we need to refresh the library (if filters are active that might change due to tag updates)
    import("./filterStore").then(({ filterState }) => {
        // If untagged filter is on, or if we have selected tags (adding/removing tags might affect visibility)
        if (filterState.filterUntagged || filterState.selectedTags.length > 0) {
            import("./libraryStore").then(({ libraryActions }) => {
                libraryActions.refreshImages(true);
            });
        }
    });
  },

  loadTags: async () => {
    try {
      const tags = await tagService.getAllTags();
      setMetadataState("tags", tags);
    } catch (err) {
      console.error("Failed to load tags:", err);
    }
  },

  loadLocations: async () => {
    try {
      const locations = await getLocations();
      setMetadataState("locations", locations);
    } catch (err) {
      console.error("Failed to load locations:", err);
    }
  },

  loadStats: async () => {
    try {
      const stats = await tagService.getLibraryStats();
      const tagMap = new Map();
      stats.tag_counts.forEach(c => tagMap.set(c.tag_id, c.count));
      
      const folderMap = new Map();
      stats.folder_counts.forEach(c => folderMap.set(c.location_id, c.count));

      setMetadataState("libraryStats", {
        total_images: stats.total_images,
        untagged_images: stats.untagged_images,
        tag_counts: tagMap,
        folder_counts: folderMap
      });
    } catch (err) {
      console.error("Failed to load library stats:", err);
    }
  },

  refreshAll: async () => {
    await Promise.all([
      metadataActions.loadTags(),
      metadataActions.loadLocations(),
      metadataActions.loadStats()
    ]);
  }
};

export { metadataState };
