/**
 * Alert component module.
 *
 * @module Alert
 * @description
 * Feedback components for displaying important messages, status, or notifications.
 * It follow the Compound Component pattern for maximum flexibility.
 *
 * @example
 * // Basic usage with props
 * <Alert.Root variant="info" title="Note" isDismissible>
 *   This is an informative message.
 * </Alert.Root>
 *
 * @example
 * // Advanced usage with compound components
 * <Alert.Root variant="success">
 *   <Alert.Title>Operation Successful</Alert.Title>
 *   <Alert.Description>
 *     The data has been synchronized with the server correctly.
 *   </Alert.Description>
 * </Alert.Root>
 */

import { AlertRoot as RootComponent } from './Root';
import { AlertTitle as TitleComponent } from './Title';
import { AlertDescription as DescriptionComponent } from './Description';

export const Alert = Object.assign(RootComponent, {
    Root: RootComponent,
    Title: TitleComponent,
    Description: DescriptionComponent
});

export * from './types';
export {
    RootComponent as AlertRoot,
    TitleComponent as AlertTitle,
    DescriptionComponent as AlertDescription
};
