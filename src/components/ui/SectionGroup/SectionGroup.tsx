import { Component, splitProps, JSX } from 'solid-js';
import { cn as concatenateClasses } from '../../../lib/utils';
import { SectionGroupProperties } from './types';
import './section-group.css';

/**
 * SectionGroup Component provides a structured container for grouping related content with a prominent header.
 * Ideal for sections in configuration panels or dashboards.
 *
 * @param {SectionGroupProperties} properties - Properties for the section grouping.
 * @returns {JSX.Element} A stylized container with a header and children components.
 *
 * @example
 * <SectionGroup title="User Profile" description="Manage your public information.">
 *   <Input label="Display name" />
 * </SectionGroup>
 */
export const SectionGroup: Component<SectionGroupProperties> = properties => {
    const [localProperties, remainingProperties] = splitProps(properties, [
        'class',
        'title',
        'children',
        'description'
    ]);

    return (
        <section
            class={concatenateClasses('section-group', localProperties.class)}
            {...(remainingProperties as JSX.HTMLAttributes<HTMLElement>)}
        >
            <div class="section-group-header">
                <h3 class="section-group-title">{localProperties.title}</h3>
                {localProperties.description && (
                    <p class="section-group-description">{localProperties.description}</p>
                )}
            </div>
            <div class="section-group-content">{localProperties.children}</div>
        </section>
    );
};
