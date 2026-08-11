import { Component, JSX } from 'solid-js';

export interface HomeViewProperties {
    /** The title bar to render at the top */
    header: JSX.Element;
}

/**
 * The Home view placeholder.
 * Will contain dashboard/overview elements in the future.
 *
 * @param {HomeViewProperties} props - Component properties.
 * @returns {JSX.Element} The rendered home view.
 */
export const HomeView: Component<HomeViewProperties> = props => {
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
                <h1 style={{ color: 'var(--text-secondary)' }}>Home</h1>
            </div>
        </div>
    );
};
