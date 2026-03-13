import { createSignal, createEffect, on, createResource } from 'solid-js';
import {
    type TranscodeQuality,
    probeVideo,
    isHlsServerAvailable,
    getVideoUrl,
    type VideoProbeResult
} from '../../lib/stream-utils';

/**
 * Custom hook to manage video source URL generation and probing.
 * Consolidates streaming strategy logic used in both ItemView and Inspector.
 */
export function useVideoSource(
    assetIdAccessor: () => string | undefined,
    pathAccessor: () => string | undefined,
    qualityAccessor: () => TranscodeQuality = () => 'standard'
) {
    const [videoUrl, setVideoUrl] = createSignal('');
    const [probeError, setProbeError] = createSignal<string | null>(null);

    // Probe video when assetId changes
    const [probeResult] = createResource(
        assetIdAccessor,
        async (id): Promise<VideoProbeResult | null> => {
            if (!id) return null;

            try {
                // First check if HLS server is available
                const serverAvailable = await isHlsServerAvailable();
                if (!serverAvailable) {
                    setProbeError(null);
                    return null;
                }

                // Probe the video using assetId
                const result = await probeVideo(id);
                setProbeError(null);
                return result;
            } catch (e) {
                console.warn('VideoSourceHook: Video probe failed:', e);
                setProbeError(e instanceof Error ? e.message : 'Probe failed');
                return null;
            }
        }
    );

    // Update URL when path, quality, or probe result changes
    createEffect(
        on(
            () => [assetIdAccessor(), pathAccessor(), qualityAccessor(), probeResult()] as const,
            ([id, path, q, probe]) => {
                if (!id || !path) {
                    setVideoUrl('');
                    return;
                }

                // Delegate URL construction to central logic in stream-utils
                const url = getVideoUrl(id, path, q, probe);
                setVideoUrl(url);
            }
        )
    );

    return {
        videoUrl,
        probeResult,
        probeError,
        get isLoading() {
            return probeResult.loading;
        }
    };
}
