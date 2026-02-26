import { Component, JSX } from 'solid-js';

/**
 * Defines the visual style variants for the Alert component.
 */
export type AlertVariant = 'default' | 'info' | 'success' | 'warning' | 'destructive';

/**
 * Properties for the AlertRoot component.
 */
export interface AlertProperties extends JSX.HTMLAttributes<HTMLDivElement> {
    /**
     * The visual variant of the alert.
     * @default 'default'
     */
    variant?: AlertVariant;
    /**
     * Optional custom icon to display.
     * If not provided, a default icon for the variant will be used.
     */
    icon?: Component<{ size?: number | string }>;
    /**
     * Optional title for the alert.
     * Can also be provided via the Alert.Title component.
     */
    title?: string;
    /**
     * Whether the alert can be dismissed by the user.
     * @default false
     */
    isDismissible?: boolean;
    /**
     * Callback function triggered when the alert is dismissed.
     */
    onDismiss?: () => void;
    /**
     * The content to be rendered inside the alert.
     */
    children?: JSX.Element;
}

/**
 * Properties for the AlertTitle component.
 */
export interface AlertTitleProperties extends JSX.HTMLAttributes<HTMLHeadingElement> {
    /**
     * The content of the title.
     */
    children: JSX.Element;
}

/**
 * Properties for the AlertDescription component.
 */
export interface AlertDescriptionProperties extends JSX.HTMLAttributes<HTMLDivElement> {
    /**
     * The content of the description.
     */
    children: JSX.Element;
}
