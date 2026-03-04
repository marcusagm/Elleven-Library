import { Component, Show } from 'solid-js';
import { Slider } from '../Slider';
import { useVideoContext } from './VideoPlayerContext';
import { formatTime } from '../../../utils/format';

/**
 * Displays the video seekbar UI including the buffered progress and preview tooltip.
 * Connects directly to `useVideoContext` internally.
 *
 * @returns Video seekbar component
 */
export const VideoSeekbar: Component = () => {
    /**
     * Gets the video context.
     * @returns The video context.
     */
    const {
        /**
         * The duration of the video.
         * @returns The duration of the video.
         */
        duration,

        /**
         * The current time of the video.
         * @returns The current time of the video.
         */
        currentTime,

        /**
         * The buffered time of the video.
         * @returns The buffered time of the video.
         */
        buffered,

        /**
         * Handles the seek event.
         * @param value - The value to seek to.
         * @returns void
         */
        handleSeek,

        /**
         * Sets the preview time.
         * @param value - The value to set the preview time to.
         * @returns void
         */
        setPreviewTime,

        /**
         * Sets the preview position.
         * @param value - The value to set the preview position to.
         * @returns void
         */
        setPreviewPos,

        /**
         * The preview time.
         * @returns The preview time.
         */
        previewTime,

        /**
         * The preview position.
         * @returns The preview position.
         */
        previewPos
    } = useVideoContext();

    /**
     * Calculates the preview time based on mouse position over the seekbar.
     *
     * @param mouseEvent - The native MouseEvent.
     * @returns void
     */
    const handleSeekMouseMove = (mouseEvent: MouseEvent) => {
        const trackElement = mouseEvent.currentTarget as HTMLElement;
        const trackBoundingRect = trackElement.getBoundingClientRect();
        const positionRatio =
            (mouseEvent.clientX - trackBoundingRect.left) / trackBoundingRect.width;
        const calculatedTime = positionRatio * duration();
        setPreviewTime(calculatedTime);
        setPreviewPos(positionRatio * 100);
    };

    return (
        <div
            class="ui-video-seekbar-container"
            onMouseMove={handleSeekMouseMove}
            onMouseLeave={() => setPreviewTime(null)}
        >
            <Show when={previewTime() !== null}>
                <div class="ui-video-seekbar-preview" style={{ left: `${previewPos()}%` }}>
                    {formatTime(previewTime()!)}
                </div>
            </Show>
            <div class="ui-video-seekbar">
                <div
                    class="ui-video-seekbar-buffer"
                    style={{ width: `${(buffered() / duration()) * 100}%` }}
                />
                <Slider
                    minimumValue={0}
                    maximumValue={duration()}
                    stepValue={0.1}
                    showTicks={false}
                    value={currentTime()}
                    onValueChange={handleSeek}
                    class="ui-video-seekbar-slider"
                />
            </div>
        </div>
    );
};
