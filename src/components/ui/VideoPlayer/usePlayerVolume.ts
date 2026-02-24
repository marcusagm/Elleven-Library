import { Accessor } from 'solid-js';
import { videoState, videoActions } from '../../../core/store/videoStore';

/**
 * Custom hook integrating audio interactions with the global `videoStore` and `audioStore`.
 * Synchronizes muting and volume adjustments directly to the video element.
 *
 * @param videoElementAccessor - Accessor for the HTML `video` element
 * @returns Volume control properties `toggleMute` and `handleVolumeChange`
 */
export function usePlayerVolume(videoElementAccessor: Accessor<HTMLVideoElement | undefined>) {
    const toggleMute = (event?: MouseEvent) => {
        if (event) {
            event.stopPropagation();
        }

        const videoElement = videoElementAccessor();
        if (!videoElement) {
            return;
        }

        const newMutedState = !videoState.isMuted();
        videoActions.setIsMuted(newMutedState);
        videoElement.muted = newMutedState;
    };

    const handleVolumeChange = (volumeValue: number) => {
        const videoElement = videoElementAccessor();
        if (!videoElement) {
            return;
        }

        const normalizedVolume = volumeValue / 100;
        videoActions.setVolume(normalizedVolume);
        videoElement.volume = normalizedVolume;

        if (normalizedVolume > 0) {
            videoActions.setIsMuted(false);
            videoElement.muted = false;
        }
    };

    return {
        toggleMute,
        handleVolumeChange
    };
}
