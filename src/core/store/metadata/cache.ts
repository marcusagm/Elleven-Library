/**
 * Persistent cache for asset metadata (EXIF, technical info).
 * Uses localStorage with TTL and size management to prevent excessive growth.
 *
 * @module MetadataCache
 */

interface CacheEntry<T> {
    data: T;
    timestamp: number;
}

const CACHE_KEY_PREFIX = 'mundam-metadata-cache:';
const DEFAULT_TTL_MS = 1000 * 60 * 60 * 24; // 24 hours
const MAX_CACHE_ENTRIES = 500;

export const metadataCache = {
    /**
     * Retrieves an item from the cache if it exists and is not expired.
     *
     * @param assetId - The unique identifier of the asset.
     * @returns The cached data or null.
     */
    get<T>(assetId: string): T | null {
        try {
            const raw = localStorage.getItem(`${CACHE_KEY_PREFIX}${assetId}`);
            if (!raw) return null;

            const entry: CacheEntry<T> = JSON.parse(raw);
            const isExpired = Date.now() - entry.timestamp > DEFAULT_TTL_MS;

            if (isExpired) {
                this.remove(assetId);
                return null;
            }

            return entry.data;
        } catch (error) {
            console.warn(`Failed to read metadata cache for ${assetId}:`, error);
            return null;
        }
    },

    /**
     * Stores an item in the cache.
     *
     * @param assetId - The unique identifier of the asset.
     * @param data - The metadata object to cache.
     */
    set<T>(assetId: string, data: T): void {
        try {
            // Clean up if we exceed the limit (simple random eviction for performance)
            this.enforceLimit();

            const entry: CacheEntry<T> = {
                data,
                timestamp: Date.now()
            };
            localStorage.setItem(`${CACHE_KEY_PREFIX}${assetId}`, JSON.stringify(entry));
        } catch (error) {
            console.warn(`Failed to set metadata cache for ${assetId}:`, error);
        }
    },

    /**
     * Removes an item from the cache.
     */
    remove(assetId: string): void {
        localStorage.removeItem(`${CACHE_KEY_PREFIX}${assetId}`);
    },

    /**
     * Wipes all cached metadata.
     */
    clear(): void {
        Object.keys(localStorage)
            .filter(key => key.startsWith(CACHE_KEY_PREFIX))
            .forEach(key => localStorage.removeItem(key));
    },

    /**
     * Ensures the number of cache entries stays within limits.
     * Removes the oldest entries if needed.
     */
    enforceLimit(): void {
        const keys = Object.keys(localStorage).filter(key => key.startsWith(CACHE_KEY_PREFIX));

        if (keys.length < MAX_CACHE_ENTRIES) return;

        // Sort by timestamp if possible, otherwise just remove some
        const entries = keys.map(key => {
            try {
                const entry = JSON.parse(localStorage.getItem(key) || '');
                return { key, timestamp: entry.timestamp };
            } catch {
                return { key, timestamp: 0 };
            }
        });

        entries.sort((a, b) => a.timestamp - b.timestamp);

        // Remove 20% of the oldest entries when limit is reached
        const toRemoveCount = Math.ceil(MAX_CACHE_ENTRIES * 0.2);
        for (let index = 0; index < toRemoveCount; index++) {
            localStorage.removeItem(entries[index].key);
        }
    }
};
