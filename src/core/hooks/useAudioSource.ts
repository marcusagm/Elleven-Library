import { createSignal, createEffect, on } from 'solid-js';
import { type TranscodeQuality, getAudioUrl } from '../../lib/stream-utils';

/**
 * Custom hook to manage audio source URL generation.
 * Consolidates streaming strategy logic used in both AudioRenderer and AudioInspector.
 */
export function useAudioSource(
    assetIdAccessor: () => string | undefined,
    pathAccessor: () => string | undefined,
    qualityAccessor: () => TranscodeQuality = () => 'standard'
) {
    const [audioUrl, setAudioUrl] = createSignal('');

    // Update URL when path or quality changes
    createEffect(
        on(
            () => [assetIdAccessor(), pathAccessor(), qualityAccessor()] as const,
            ([id, path, q]) => {
                if (!id || !path) {
                    setAudioUrl('');
                    return;
                }

                // Delegate URL construction to central logic in stream-utils
                const url = getAudioUrl(id, path, q);
                setAudioUrl(url);
            }
        )
    );

    return {
        audioUrl
    };
}
