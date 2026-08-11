import { Component, JSX } from 'solid-js';
import { AppShell } from '../layouts/AppShell';
import { LibrarySidebar } from '../components/layout/LibrarySidebar';
import { FileInspector } from '../components/layout/FileInspector';
import { GlobalStatusbar } from '../components/layout/GlobalStatusbar';
import { Viewport } from '../components/layout/Viewport';

export interface GalleryViewProperties {
    /** The title bar to render at the top */
    header: JSX.Element;
}

/**
 * The main Asset Gallery view.
 * Uses the AppShell layout with a left library sidebar, central viewport,
 * right inspector, and global statusbar.
 *
 * @param {GalleryViewProperties} props - Component properties.
 * @returns {JSX.Element} The rendered gallery view.
 */
export const GalleryView: Component<GalleryViewProperties> = props => {
    return (
        <AppShell
            header={props.header}
            sidebar={<LibrarySidebar />}
            inspector={<FileInspector />}
            statusbar={<GlobalStatusbar />}
        >
            <Viewport />
        </AppShell>
    );
};
