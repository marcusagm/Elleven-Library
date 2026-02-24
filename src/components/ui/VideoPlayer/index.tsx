import { Component, Show } from 'solid-js';
import { cn } from '../../../lib/utils';
import { Button } from '../Button';
import { Loader } from '../Loader';
import { Play, Pause, AlertCircle } from 'lucide-solid';
import { videoState } from '../../../core/store/videoStore';
import { VideoProvider, useVideoContext } from './VideoPlayerContext';
import { VideoControls } from './VideoControls';
import { VideoPlayerProps } from './types';
import './video-player.css';

const VideoPlayerContent: Component = () => {
    const {
        props,
        setVideoElement,
        setContainerElement,
        isPlaying,
        togglePlay,
        toggleFullscreen,
        toggleMute,
        isLoading,
        setIsLoading,
        isTranscoding,
        error,
        setError,
        resetControlsTimeout,
        handlePlayParams,
        handlePause,
        handleTimeUpdate,
        updateBuffered,
        handleLoadedMetadata,
        handleVolumeChange,
        skip,
        lastAction,
        showControls,
        setRetryCount,
        handleError,
        getStreamMode
    } = useVideoContext();

    const getModeDotClass = () => {
        switch (getStreamMode()) {
            case 'LIVE':
                return 'ui-video-badge-live';
            case 'HLS':
                return 'ui-video-badge-hls';
            case 'NATIVE':
                return 'ui-video-badge-native';
            default:
                return 'ui-video-badge-unknown';
        }
    };

    const handleKeyDown = (e: KeyboardEvent) => {
        resetControlsTimeout();

        const actionMap: Record<string, () => void> = {
            Space: togglePlay,
            KeyK: togglePlay,
            ArrowLeft: () => skip(-5),
            KeyJ: () => skip(-5),
            ArrowRight: () => skip(5),
            KeyL: () => skip(5),
            ArrowUp: () => handleVolumeChange(Math.min((videoState.volume() + 0.1) * 100, 100)),
            ArrowDown: () => handleVolumeChange(Math.max((videoState.volume() - 0.1) * 100, 0)),
            KeyF: toggleFullscreen,
            KeyM: toggleMute
        };

        const action = actionMap[e.code];
        if (action) {
            e.preventDefault();
            action();
        }
    };

    return (
        <div
            ref={setContainerElement}
            class={cn(
                'ui-video-player',
                props.variant === 'compact' && 'ui-video-player-compact',
                props.class
            )}
            onMouseMove={resetControlsTimeout}
            onMouseEnter={resetControlsTimeout}
            onContextMenu={resetControlsTimeout}
            onKeyDown={handleKeyDown}
            tabindex="0"
        >
            <Show when={isTranscoding()}>
                <div class="ui-video-transcoding">
                    <Loader size="lg" />
                    <p>Transcoding video...</p>
                    <p class="ui-video-transcoding-hint">This may take a while for large files</p>
                </div>
            </Show>

            <Show when={error() && !isTranscoding()}>
                <div class="ui-video-error">
                    <AlertCircle size={48} />
                    <p>{error()}</p>
                    <Button
                        variant="outline"
                        onClick={() => {
                            setError(null);
                            setRetryCount(0);
                        }}
                    >
                        Retry
                    </Button>
                </div>
            </Show>

            <Show when={!error()}>
                <video
                    ref={setVideoElement}
                    src={props.src}
                    class="ui-video-element"
                    autoplay={props.autoPlay}
                    preload="auto"
                    onPlay={handlePlayParams}
                    onPause={handlePause}
                    onTimeUpdate={() => {
                        handleTimeUpdate();
                    }}
                    onProgress={updateBuffered}
                    onSeeking={updateBuffered}
                    onSeeked={updateBuffered}
                    onLoadedMetadata={handleLoadedMetadata}
                    onWaiting={() => setIsLoading(true)}
                    onPlaying={() => setIsLoading(false)}
                    onEnded={props.onEnded}
                    onClick={e => togglePlay(e)}
                    onDblClick={e => toggleFullscreen(e)}
                    onError={handleError}
                    tabindex="-1"
                />

                <Show when={isLoading()}>
                    <div class="ui-video-loader">
                        <Loader size="lg" />
                    </div>
                </Show>

                <div class="ui-video-center-action">
                    <Show when={lastAction() === 'play'}>
                        <div class="ui-video-center-action-pulse">
                            <Play size={48} fill="currentColor" />
                        </div>
                    </Show>
                    <Show when={lastAction() === 'pause'}>
                        <div class="ui-video-center-action-pulse">
                            <Pause size={48} fill="currentColor" />
                        </div>
                    </Show>
                </div>

                <div class="ui-video-badge">
                    <div class={cn('ui-video-badge-dot', getModeDotClass())} />
                    {getStreamMode()}
                </div>

                <div
                    class={cn(
                        'ui-video-controls',
                        !isPlaying() && 'ui-video-controls-visible',
                        showControls() && 'ui-video-controls-visible'
                    )}
                >
                    <Show when={props.title && props.variant === 'full'}>
                        <div class="ui-video-top-bar">{props.title}</div>
                    </Show>

                    <VideoControls />
                </div>
            </Show>
        </div>
    );
};

/**
 * VideoPlayer Component
 *
 * High-performance, fully featured video player built for Solid.js and Tauri.
 * - Manages Custom UI Controls and HLS/Native playback.
 * - Syncs volume and activity state with global `audioStore` and `videoStore`.
 *
 * @param props - Configuration properties `VideoPlayerProps`
 * @returns Video player UI node
 */
export const VideoPlayer: Component<VideoPlayerProps> = props => {
    return (
        <VideoProvider {...props}>
            <VideoPlayerContent />
        </VideoProvider>
    );
};
