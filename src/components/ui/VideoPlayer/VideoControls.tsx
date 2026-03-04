import { Component, Show, For, JSX } from 'solid-js';
import { cn } from '../../../lib/utils';
import { Button } from '../Button';
import { Slider } from '../Slider';
import { Tooltip } from '../Tooltip';
import { Popover } from '../Popover';
import {
    Play,
    Pause,
    Volume2,
    VolumeX,
    Maximize,
    Minimize,
    SkipBack,
    SkipForward,
    Settings,
    Check
} from 'lucide-solid';
import { videoState } from '../../../core/store/videoStore';
import { useVideoContext } from './VideoPlayerContext';
import { formatTime } from '../../../utils/format';
import { QUALITY_OPTIONS } from './types';
import { VideoSeekbar } from './VideoSeekbar';

/**
 * Displays the video UI controls including playback, volume, and fullscreen interactions.
 * Connects directly to `useVideoContext` internally.
 *
 * @returns {JSX.Element} Video player controls component
 */
export const VideoControls: Component = (): JSX.Element => {
    /**
     * Accessor for the video player properties
     *
     * @returns {Object} The video player properties.
     */
    const {
        /**
         * The video player properties
         *
         * @returns {Object} The video player properties.
         */
        props,

        /**
         * The video player properties
         *
         * @returns {Object} The video player properties.
         */
        isPlaying,

        /**
         * The video player properties
         *
         * @returns {Object} The video player properties.
         */
        togglePlay,

        /**
         * The video player properties
         *
         * @returns {Object} The video player properties.
         */
        skip,

        /**
         * The video player properties
         *
         * @returns {Object} The video player properties.
         */
        toggleMute,

        /**
         * The video player properties
         *
         * @returns {Object} The video player properties.
         */
        handleVolumeChange,

        /**
         * The video player properties
         *
         * @returns {Object} The video player properties.
         */
        currentTime,

        /**
         * The video player properties
         *
         * @returns {Object} The video player properties.
         */
        duration,

        /**
         * The video player properties
         *
         * @returns {Object} The video player properties.
         */
        cyclePlaybackRate,

        /**
         * The video player properties
         *
         * @returns {Object} The video player properties.
         */
        isFullscreen,

        /**
         * The video player properties
         *
         * @returns {Object} The video player properties.
         */
        toggleFullscreen,

        /**
         * The video player properties
         *
         * @returns {Object} The video player properties.
         */
        needsTranscode
    } = useVideoContext();

    return (
        <div class="ui-video-bottom-controls">
            {/* Seek Bar Area */}
            <VideoSeekbar />

            <div class="ui-video-controls-row">
                <div class="ui-video-controls-left ui-video-controls-row">
                    <Tooltip content={isPlaying() ? 'Pause' : 'Play'}>
                        <Button
                            variant="ghost"
                            size="icon-sm"
                            onClick={(mouseEvent: MouseEvent) => togglePlay(mouseEvent)}
                        >
                            <Show
                                when={isPlaying()}
                                fallback={<Play size={18} fill="currentColor" />}
                            >
                                <Pause size={18} fill="currentColor" />
                            </Show>
                        </Button>
                    </Tooltip>

                    <Show when={props.variant === 'full'}>
                        <Tooltip content="Step backward 5s">
                            <Button variant="ghost" size="icon-sm" onClick={() => skip(-5)}>
                                <SkipBack size={18} fill="currentColor" />
                            </Button>
                        </Tooltip>
                        <Tooltip content="Step forward 5s">
                            <Button variant="ghost" size="icon-sm" onClick={() => skip(5)}>
                                <SkipForward size={18} fill="currentColor" />
                            </Button>
                        </Tooltip>
                    </Show>

                    <div class="ui-video-volume-group">
                        <Button
                            variant="ghost"
                            size="icon-sm"
                            onClick={(mouseEvent: MouseEvent) => toggleMute(mouseEvent)}
                        >
                            <Show
                                when={videoState.isMuted() || videoState.volume() === 0}
                                fallback={<Volume2 size={18} />}
                            >
                                <VolumeX size={18} />
                            </Show>
                        </Button>
                        <div class="ui-video-volume-slider">
                            <Slider
                                minimumValue={0}
                                maximumValue={100}
                                showTicks={false}
                                value={videoState.isMuted() ? 0 : videoState.volume() * 100}
                                onValueChange={handleVolumeChange}
                            />
                        </div>
                    </div>

                    <div class="ui-video-time">
                        {formatTime(currentTime())} <span>/</span> {formatTime(duration())}
                    </div>
                </div>

                <div style={{ flex: 1 }} />

                <div class="ui-video-controls-right ui-video-controls-row">
                    <Show when={props.variant === 'full'}>
                        <Tooltip content="Playback Speed">
                            <Button
                                variant="ghost"
                                size="sm"
                                class="ui-video-speed-btn"
                                onClick={mouseEvent => cyclePlaybackRate(mouseEvent)}
                            >
                                {videoState.playbackRate()}x
                            </Button>
                        </Tooltip>
                    </Show>

                    {/* Quality Selector - only for transcoded videos when enabled */}
                    <Show
                        when={
                            props.showQualitySelector !== false &&
                            needsTranscode() &&
                            props.onQualityChange
                        }
                    >
                        <Popover
                            trigger={
                                <Tooltip content="Quality">
                                    <Button
                                        variant="ghost"
                                        size="icon-sm"
                                        class="ui-video-quality-btn"
                                    >
                                        <Settings size={18} />
                                    </Button>
                                </Tooltip>
                            }
                            align="end"
                        >
                            <div class="ui-video-quality-menu">
                                <div class="ui-video-quality-title">Quality</div>
                                <For each={QUALITY_OPTIONS}>
                                    {option => (
                                        <button
                                            class={cn(
                                                'ui-video-quality-option',
                                                (props.quality || 'standard') === option.id &&
                                                    'ui-video-quality-option-active'
                                            )}
                                            onClick={() => props.onQualityChange?.(option.id)}
                                        >
                                            <span>{option.label}</span>
                                            <Show
                                                when={(props.quality || 'standard') === option.id}
                                            >
                                                <Check size={14} />
                                            </Show>
                                        </button>
                                    )}
                                </For>
                            </div>
                        </Popover>
                    </Show>

                    <Tooltip content={isFullscreen() ? 'Exit Fullscreen' : 'Fullscreen'}>
                        <Button
                            variant="ghost"
                            size="icon-sm"
                            onClick={mouseEvent => toggleFullscreen(mouseEvent)}
                        >
                            <Show when={isFullscreen()} fallback={<Maximize size={18} />}>
                                <Minimize size={18} />
                            </Show>
                        </Button>
                    </Tooltip>
                </div>
            </div>
        </div>
    );
};
