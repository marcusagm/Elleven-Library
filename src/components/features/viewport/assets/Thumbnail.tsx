import { createSignal, createMemo, Show, onMount, onCleanup } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import { Loader } from '../../../ui/Loader';
import {
    isPendingRegeneration,
    markPendingRegeneration,
    getCompletedThumbnail,
    clearCompleted,
    subscribeThumbnailReady
} from '../../../../core/store/thumbnailStore';
import './thumbnail.css';

/**
 * Tracks which thumbnail URLs have been successfully loaded.
 * This persists across component recycling during virtualization.
 */
const loadedThumbnails = new Set<string>();

/**
 * Adds a loaded thumbnail to cache.
 *
 * @param {string} url - The URL to mark as loaded.
 */
function markThumbnailLoaded(url: string) {
    loadedThumbnails.add(url);
}

/**
 * Checks if a thumbnail was previously loaded.
 *
 * @param {string | undefined} url - The URL to check.
 * @returns {boolean} True if loaded, otherwise false.
 */
function isThumbnailLoaded(url: string | undefined): boolean {
    return url ? loadedThumbnails.has(url) : false;
}

/**
 * Represents the configuration for the Thumbnail component.
 */
export interface ThumbnailProperties {
    /**
     * The unique identifier of the asset.
     */
    id: string;
    /**
     * The original source file path or URL.
     */
    src: string;
    /**
     * The thumbnail file path or URL.
     */
    thumbnail: string | null;
    /**
     * The alternative text for the image.
     */
    alt: string;
    /**
     * The optional width of the thumbnail.
     */
    width?: number | null;
    /**
     * The optional height of the thumbnail.
     */
    height?: number | null;
}

/**
 * Renders an optimized asset thumbnail with loading states, lazy loading cache, and regeneration routines.
 *
 * @param {ThumbnailProperties} thumbnailProperties - Properties for the thumbnail display.
 * @returns {JSX.Element} The rendered image or a loading placeholder.
 *
 * @example
 * ```tsx
 * import { Thumbnail } from '@/components/features/viewport/Thumbnail';
 * <Thumbnail id={1} src="/path/to/img.png" thumbnail="/path/to/thumb" alt="My image" width={200} height={200} />
 * ```
 */
export function Thumbnail(thumbnailProperties: ThumbnailProperties) {
    /**
     * Local error state for the thumbnail.
     *
     * @returns {boolean} The local error state.
     */
    const [localError, setLocalError] = createSignal(false);

    /**
     * Local thumbnail state for the thumbnail.
     *
     * @returns {string | null} The local thumbnail state.
     */
    const [localThumbnail, setLocalThumbnail] = createSignal<string | null>(null);

    /**
     * Unsubscribe function for the thumbnail ready event.
     *
     * @returns {(() => void) | null} The unsubscribe function.
     */
    let unsubscribe: (() => void) | null = null;

    /**
     * Mount lifecycle for the thumbnail component.
     *
     * @returns {void}
     */
    onMount(() => {
        unsubscribe = subscribeThumbnailReady(
            thumbnailProperties.id,
            (_id: string, path: string) => {
                setLocalThumbnail(path);
                setLocalError(false);
            }
        );
    });

    /**
     * Cleanup lifecycle for the thumbnail component.
     *
     * @returns {void}
     */
    onCleanup(() => {
        if (unsubscribe) unsubscribe();
    });

    /**
     * Effective thumbnail for the thumbnail component.
     *
     * @returns {string | null} The effective thumbnail.
     */
    const effectiveThumbnail = createMemo(() => {
        const local = localThumbnail();
        if (local) return local;

        const completed = getCompletedThumbnail(thumbnailProperties.id);
        if (completed) {
            clearCompleted(thumbnailProperties.id);
            return completed;
        }

        return thumbnailProperties.thumbnail;
    });

    /**
     * Should show image for the thumbnail component.
     *
     * @returns {boolean} True if the image should be shown, otherwise false.
     */
    const shouldShowImage = createMemo(() => {
        if (
            isPendingRegeneration(thumbnailProperties.id) &&
            !localThumbnail() &&
            !getCompletedThumbnail(thumbnailProperties.id)
        ) {
            return false;
        }
        return true;
    });

    /**
     * Thumbnail URL for the thumbnail component.
     *
     * @returns {string | undefined} The thumbnail URL.
     */
    const thumbUrl = createMemo(() => {
        const path = effectiveThumbnail();
        if (!path || path === '') return undefined;

        return `asset://localhost/${thumbnailProperties.id}?type=thumb`;
    });

    /**
     * Display source for the thumbnail component.
     *
     * @returns {string | undefined} The display source.
     */
    const displaySrc = createMemo((): string | undefined => {
        if (localError()) return undefined;
        if (!shouldShowImage()) return undefined;
        return thumbUrl();
    });

    /**
     * Is already loaded for the thumbnail component.
     *
     * @returns {boolean} True if the thumbnail is already loaded, otherwise false.
     */
    const isAlreadyLoaded = createMemo(() => isThumbnailLoaded(thumbUrl()));

    /**
     * Loaded state for the thumbnail component.
     *
     * @returns {boolean} True if the thumbnail is loaded, otherwise false.
     */
    const [loaded, setLoaded] = createSignal(false);

    /**
     * Aspect ratio for the thumbnail component.
     *
     * @returns {string | undefined} The aspect ratio.
     */
    const aspectRatio = createMemo(() => {
        if (thumbnailProperties.width && thumbnailProperties.height) {
            return `${thumbnailProperties.width} / ${thumbnailProperties.height}`;
        }
        return undefined;
    });

    /**
     * Handle load for the thumbnail component.
     *
     * @returns {void}
     */
    const handleLoad = () => {
        const url = thumbUrl();
        if (url) {
            markThumbnailLoaded(url);
        }
        setLoaded(true);
    };

    /**
     * Number of times we have attempted to regenerate this thumbnail in the current session.
     */
    const [retryCount, setRetryCount] = createSignal(0);
    const MAX_RETRIES = 2;

    /**
     * Handle error for the thumbnail component.
     *
     * @returns {void}
     */
    const handleError = () => {
        const thumb = thumbUrl();
        if (!thumb) return;

        // If we've already marked an error or are at the retry limit, stop to prevent loops.
        if (localError() || retryCount() >= MAX_RETRIES) {
            setLocalError(true);
            return;
        }

        // If another component already requested regeneration, we just wait.
        if (isPendingRegeneration(thumbnailProperties.id)) {
            setLocalError(true);
            return;
        }

        setLocalError(true);
        setRetryCount(prev => prev + 1);
        markPendingRegeneration(thumbnailProperties.id);

        invoke('request_thumbnail_regenerate', { assetId: thumbnailProperties.id }).catch(error =>
            console.error('Failed to request regeneration:', error)
        );
    };

    /**
     * Show placeholder for the thumbnail component.
     *
     * @returns {boolean} True if the placeholder should be shown, otherwise false.
     */
    const showPlaceholder = createMemo(() => {
        if (!displaySrc()) return true;
        if (localError()) return true;
        if (isAlreadyLoaded()) return false;
        if (!loaded()) return true;
        return false;
    });

    return (
        <div
            class="thumbnail-container"
            style={{ 'aspect-ratio': aspectRatio() }}
            data-id={thumbnailProperties.id}
        >
            <Show when={showPlaceholder()}>
                <div class="asset-placeholder">
                    <Loader size="sm" />
                </div>
            </Show>

            <Show when={displaySrc() && !localError()}>
                <img
                    src={displaySrc()}
                    alt={thumbnailProperties.alt}
                    draggable={false}
                    onLoad={handleLoad}
                    onError={handleError}
                    class={loaded() || isAlreadyLoaded() ? 'loaded' : 'loading'}
                />
            </Show>
        </div>
    );
}
