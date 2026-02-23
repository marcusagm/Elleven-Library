/**
 * HLS Player Manager
 *
 * Manages the lifecycle of an hls.js instance attached to a video element.
 */

import Hls from 'hls.js';
import { type HlsPlayerOptions } from './hls-player';

/** Default options for HLS player */
const DEFAULT_OPTIONS: Required<HlsPlayerOptions> = {
    debug: false,
    autoStartLoad: true,
    seekDebounceMs: 150
};

export class HlsPlayerManager {
    private hlsInstance: Hls | null = null;
    private mediaElement: HTMLMediaElement | null = null;
    private mergedOptions: Required<HlsPlayerOptions>;
    private seekDebounceTimeout: ReturnType<typeof setTimeout> | null = null;

    constructor(options: HlsPlayerOptions = {}) {
        this.mergedOptions = { ...DEFAULT_OPTIONS, ...options };
    }

    /** Check if HLS is supported in the current browser */
    static isSupported(): boolean {
        return Hls.isSupported();
    }

    /** Attach the HLS player to a media element and load a playlist */
    attach(mediaElement: HTMLMediaElement, playlistUrl: string): void {
        this.detach();
        this.mediaElement = mediaElement;

        // Safari has native HLS support
        if (mediaElement.canPlayType('application/vnd.apple.mpegurl')) {
            mediaElement.src = playlistUrl;
            return;
        }

        if (!Hls.isSupported()) {
            console.error('HLS is not supported in this browser');
            return;
        }

        this.hlsInstance = new Hls({
            debug: this.mergedOptions.debug,
            autoStartLoad: this.mergedOptions.autoStartLoad,
            maxBufferLength: 30,
            maxMaxBufferLength: 60,
            maxBufferSize: 60 * 1024 * 1024,
            fragLoadingMaxRetry: 3,
            manifestLoadingMaxRetry: 3,
            levelLoadingMaxRetry: 3
        });

        this.hlsInstance.attachMedia(mediaElement);

        this.hlsInstance.on(Hls.Events.MEDIA_ATTACHED, () => {
            this.hlsInstance?.loadSource(playlistUrl);
        });

        this.hlsInstance.on(Hls.Events.ERROR, (_event, data) => {
            if (data.fatal) {
                this.handleFatalError(data.type);
            }
        });
    }

    /** Handle fatal HLS errors with recovery attempts */
    private handleFatalError(errorType: string): void {
        switch (errorType) {
            case Hls.ErrorTypes.NETWORK_ERROR:
                console.error('HLS network error, trying to recover...');
                this.hlsInstance?.startLoad();
                break;
            case Hls.ErrorTypes.MEDIA_ERROR:
                console.error('HLS media error, trying to recover...');
                this.hlsInstance?.recoverMediaError();
                break;
            default:
                console.error('HLS fatal error, destroying...');
                this.destroy();
                break;
        }
    }

    /** Detach the HLS player from the media element */
    detach(): void {
        if (this.seekDebounceTimeout) {
            clearTimeout(this.seekDebounceTimeout);
            this.seekDebounceTimeout = null;
        }

        if (this.hlsInstance) {
            this.hlsInstance.detachMedia();
        }

        if (this.mediaElement) {
            this.mediaElement.removeAttribute('src');
            this.mediaElement.load();
            this.mediaElement = null;
        }
    }

    /** Destroy the HLS player instance */
    destroy(): void {
        this.detach();

        if (this.hlsInstance) {
            this.hlsInstance.destroy();
            this.hlsInstance = null;
        }
    }

    /** Start loading the stream */
    startLoad(startPosition?: number): void {
        this.hlsInstance?.startLoad(startPosition ?? -1);
    }

    /** Stop loading the stream */
    stopLoad(): void {
        this.hlsInstance?.stopLoad();
    }

    /** Debounced seek — useful for scrubbing */
    debouncedSeek(time: number): void {
        if (this.seekDebounceTimeout) {
            clearTimeout(this.seekDebounceTimeout);
        }

        this.seekDebounceTimeout = setTimeout(() => {
            if (this.mediaElement) {
                this.mediaElement.currentTime = time;
            }
        }, this.mergedOptions.seekDebounceMs);
    }

    /** Get the underlying hls.js instance */
    getHls(): Hls | null {
        return this.hlsInstance;
    }

    /** Get the media element */
    getMediaElement(): HTMLMediaElement | null {
        return this.mediaElement;
    }
}
