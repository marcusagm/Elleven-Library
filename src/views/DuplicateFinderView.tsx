import { Component, JSX } from 'solid-js';

export interface DuplicateFinderViewProperties {
    /** The title bar to render at the top */
    header: JSX.Element;
}

/**
 * The Duplicate Finder view placeholder.
 * Will contain tools for finding and resolving duplicate assets in the future.
 *
 * @param {DuplicateFinderViewProperties} props - Component properties.
 * @returns {JSX.Element} The rendered duplicate finder view.
 */
export const DuplicateFinderView: Component<DuplicateFinderViewProperties> = props => {
    return (
        <div
            style={{
                display: 'flex',
                'flex-direction': 'column',
                height: '100vh',
                width: '100vw',
                overflow: 'hidden'
            }}
        >
            {props.header}
            <div
                style={{
                    flex: 1,
                    display: 'flex',
                    'align-items': 'center',
                    'justify-content': 'center'
                }}
            >
                <h1 style={{ color: 'var(--text-secondary)' }}>Duplicate Files</h1>
            </div>
        </div>
    );
};
