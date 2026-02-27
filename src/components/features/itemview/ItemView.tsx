import {
    Component,
    createMemo,
    Show,
    Switch,
    Match,
    createEffect,
    onCleanup,
    onMount,
    createSignal
} from 'solid-js';
import { useViewport, useLibrary } from '../../../core/hooks';
import { useCommands, createConditionalScope } from '../../../core/input';
import { ItemViewProvider, useItemViewContext, FlipState } from './ItemViewContext';
import { BaseToolbar } from './common/BaseToolbar';
import { ImageToolbar } from './renderers/image/ImageToolbar';
import { FontToolbar } from './renderers/font/FontToolbar';
import { ImageViewer } from './renderers/image/ImageViewer';
import { VideoPlayer } from './renderers/video/VideoPlayer';
import { FontView } from './renderers/font/FontView';
import { ModelViewer } from './renderers/model/ModelViewer';
import { ModelToolbar } from './renderers/model/ModelToolbar';
import { AudioRenderer } from './renderers/audio/AudioRenderer';
import { Loader } from '../../ui/Loader';
import { getMediaType } from '../../../lib/stream-utils';
import './item-view.css';

export const ItemView: Component = () => {
    return (
        <ItemViewProvider>
            <ItemViewContent />
        </ItemViewProvider>
    );
};

const ItemViewContent: Component = () => {
    const viewport = useViewport();
    const lib = useLibrary();
    const {
        reset,
        setMediaType,
        mediaType,
        slideshowPlaying,
        slideshowDuration,
        setSlideshowPlaying,
        setTool,
        setFlip
    } = useItemViewContext();

    let overlayRef: HTMLDivElement | undefined;
    let previousFocus: HTMLElement | null = null;

    onMount(() => {
        previousFocus = document.activeElement as HTMLElement;
        // Focus the overlay to trap keyboard events and prevent background scrolling
        requestAnimationFrame(() => {
            overlayRef?.focus();
        });
    });

    onCleanup(() => {
        // Restore focus to the grid/list item
        previousFocus?.focus();
    });

    // Push image-viewer scope with blocking enabled (isolates input)
    createConditionalScope('image-viewer', () => true, undefined, true);

    const [itemLoading, setItemLoading] = createSignal(false);
    const item = createMemo(() => lib.items.find(i => i.id.toString() === viewport.activeItemId()));

    // Reset view state when item changes
    createEffect(() => {
        const i = item();
        if (i) {
            setItemLoading(true);
            reset();
            const type = getMediaType(i.filename);
            setMediaType(type);
            // Small delay to allow renderer to mount/unmount smoothly
            setTimeout(() => setItemLoading(false), 300);
        }
    });

    const navigate = (direction: number) => {
        const items = lib.items;
        const currentId = viewport.activeItemId();
        const currentIndex = items.findIndex(i => i.id.toString() === currentId);

        if (currentIndex !== -1) {
            const nextIndex = (currentIndex + direction + items.length) % items.length;
            viewport.openItem(items[nextIndex].id.toString());
        }
    };

    // Slideshow Timer Logic
    createEffect(() => {
        if (slideshowPlaying() && item()) {
            const durationMs = slideshowDuration() * 1000;
            const interval = setInterval(() => {
                navigate(1);
            }, durationMs);

            onCleanup(() => clearInterval(interval));
        }
    });

    // Stop slideshow on unmount (closing viewer)
    onCleanup(() => {
        setSlideshowPlaying(false);
    });

    const zoomIn = () => viewport.setZoom(Math.min(viewport.zoom() + 10, 500));
    const zoomOut = () => viewport.setZoom(Math.max(viewport.zoom() - 10, 5));
    const fitToScreen = () => window.dispatchEvent(new CustomEvent('viewport:fit'));
    const originalSize = () => viewport.setZoom(100);
    const toggleFlipH = () => setFlip((f: FlipState) => ({ ...f, horizontal: !f.horizontal }));
    const toggleFlipV = () => setFlip((f: FlipState) => ({ ...f, vertical: !f.vertical }));

    // Global navigation shortcuts (ItemView level)
    // Global navigation shortcuts (ItemView level) using centralized commands
    useCommands({
        'viewer:close': () => viewport.closeItem(),
        'viewer:zoom-in': zoomIn,
        'viewer:zoom-out': zoomOut,
        'viewer:fit-screen': fitToScreen,
        'viewer:original-size': originalSize,
        'viewer:tool-pan': () => setTool('pan'),
        'viewer:tool-rotate': () => setTool('rotate'),
        'viewer:previous': () => navigate(-1),
        'viewer:next': () => navigate(1),
        'viewer:slideshow-toggle': () => setSlideshowPlaying(!slideshowPlaying()),
        'viewer:flip-h': toggleFlipH,
        'viewer:flip-v': toggleFlipV
    });

    return (
        <div
            ref={overlayRef}
            class="item-view-overlay"
            tabIndex={-1}
            style={{ outline: 'none' }}
            role="dialog"
            aria-modal="true"
        >
            <BaseToolbar>
                <Switch>
                    <Match
                        when={
                            mediaType() === 'image' ||
                            mediaType() === 'unknown' ||
                            mediaType() === 'project' ||
                            mediaType() === 'archive'
                        }
                    >
                        <ImageToolbar />
                    </Match>
                    <Match when={mediaType() === 'font'}>
                        <FontToolbar />
                    </Match>
                    <Match when={mediaType() === 'model3d'}>
                        <ModelToolbar />
                    </Match>
                </Switch>
            </BaseToolbar>

            <Show when={item()} fallback={<div class="item-error">Asset not found</div>}>
                <div class="item-renderer-wrapper" classList={{ 'is-changing': itemLoading() }}>
                    <Show when={itemLoading()}>
                        <div class="item-switch-loader">
                            <Loader size="lg" text="Loading asset..." />
                        </div>
                    </Show>

                    <Switch fallback={<div class="item-error">Unsupported format</div>}>
                        <Match
                            when={
                                getMediaType(item()!.filename) === 'image' ||
                                getMediaType(item()!.filename) === 'unknown' ||
                                getMediaType(item()!.filename) === 'project' ||
                                getMediaType(item()!.filename) === 'archive'
                            }
                        >
                            <ImageViewer
                                src={`image://localhost/${encodeURIComponent(item()!.path)}`}
                                alt={item()!.filename}
                            />
                        </Match>
                        <Match when={getMediaType(item()!.filename) === 'video'}>
                            <VideoPlayer path={item()!.path} />
                        </Match>
                        <Match when={getMediaType(item()!.filename) === 'audio'}>
                            <AudioRenderer path={item()!.path} />
                        </Match>
                        <Match when={mediaType() === 'font'}>
                            <FontView
                                src={`font://localhost/${encodeURIComponent(item()!.path)}`}
                                fontName={item()!.filename}
                            />
                        </Match>
                        <Match when={getMediaType(item()!.filename) === 'model3d'}>
                            <ModelViewer
                                src={`model://localhost/${encodeURIComponent(item()!.path)}`}
                                filename={item()!.filename}
                                thumbnail={item()!.thumbnail_path}
                            />
                        </Match>
                    </Switch>
                </div>
            </Show>
        </div>
    );
};
