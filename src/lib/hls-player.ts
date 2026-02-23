/**
 * HLS Player Utilities
 *
 * Types, constants, and utility functions for HLS streaming integration.
 * For the player manager class, see hls-manager.ts
 * For the SolidJS hook, see createHlsPlayer.ts
 */

import { fetch } from '@tauri-apps/plugin-http';

export interface HlsPlayerOptions {
    /** Enable debug logging */
    debug?: boolean;
    /** Start loading immediately on attach */
    autoStartLoad?: boolean;
    /** Debounce delay for seek operations (ms) */
    seekDebounceMs?: number;
}

export interface HlsPlayerState {
    /** Whether the player is loading */
    isLoading: boolean;
    /** Whether the player has encountered an error */
    hasError: boolean;
    /** Error message if any */
    errorMessage: string | null;
    /** Current buffered percentage */
    buffered: number;
}

/** HLS streaming server base URL */
export const HLS_SERVER_URL = 'http://127.0.0.1:9876';

/**
 * Get the HLS playlist URL for a video file
 * @param filePath - Absolute path to the video file
 * @returns The M3U8 playlist URL
 */
export function getHlsPlaylistUrl(filePath: string, quality: string = 'standard'): string {
    const encodedPath = encodeURIComponent(filePath);
    return `${HLS_SERVER_URL}/playlist/${encodedPath}?quality=${quality}`;
}

/**
 * Get the probe URL for a video file
 * @param filePath - Absolute path to the video file
 * @returns The probe endpoint URL
 */
export function getHlsProbeUrl(filePath: string): string {
    const encodedPath = encodeURIComponent(filePath);
    return `${HLS_SERVER_URL}/probe/${encodedPath}`;
}

/**
 * Probe a video file to get metadata and native format detection
 */
export interface VideoProbeResult {
    duration_secs: number;
    is_native: boolean;
    video_codec: string | null;
    audio_codec: string | null;
    container: string | null;
    width: number | null;
    height: number | null;
}

/**
 * Probe a video file for metadata
 * @param filePath - Absolute path to the video file
 * @returns Video metadata including duration and native format detection
 */
export async function probeVideo(filePath: string): Promise<VideoProbeResult> {
    const url = getHlsProbeUrl(filePath);
    const response = await fetch(url);

    if (!response.ok) {
        throw new Error(`Probe failed: ${response.statusText}`);
    }

    return response.json();
}

/**
 * Check if HLS streaming server is available
 */
export async function isHlsServerAvailable(): Promise<boolean> {
    try {
        const response = await fetch(`${HLS_SERVER_URL}/health`, {
            method: 'GET'
        });
        return response.ok;
    } catch {
        return false;
    }
}

/**
 * Check if a URL is an HLS playlist
 */
export function isHlsUrl(url: string): boolean {
    return url.endsWith('.m3u8') || url.includes(HLS_SERVER_URL);
}

// Re-export for backwards compatibility
export { HlsPlayerManager } from './hls-manager';
export { createHlsPlayer } from './createHlsPlayer';
