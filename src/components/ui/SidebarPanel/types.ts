import { JSX } from 'solid-js';

/**
 * Properties for the SidebarPanel component.
 */
export interface SidebarPanelProperties extends JSX.HTMLAttributes<HTMLElement> {
    /**
     * The title displayed at the top of the sidebar panel.
     */
    title: string;
    /**
     * The main body content for the panel.
     */
    children: JSX.Element;
    /**
     * Optional elements (like buttons) to display in the header next to the title.
     */
    headerActions?: JSX.Element;
    /**
     * Optional content to display in the panel's footer.
     */
    footerContent?: JSX.Element;
    /**
     * Additional CSS class applied directly to the content body.
     */
    contentClass?: string;
}
