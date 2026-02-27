import { z } from 'zod';
import { LayoutMode } from './types';

// ============================================================================
// Layout Input Schemas
// ============================================================================

export const LayoutItemInputSchema = z.object({
    id: z.number().int().positive(),
    aspectRatio: z.number().positive()
});

export const LayoutModeSchema = z.enum([
    'masonry',
    'masonry-v',
    'masonry-h',
    'grid'
]) as z.ZodType<LayoutMode>;

export const LayoutConfigSchema = z.object({
    mode: LayoutModeSchema,
    containerWidth: z.number().nonnegative(),
    itemSize: z.number().positive(),
    gap: z.number().nonnegative(),
    buffer: z.number().nonnegative()
});

// ============================================================================
// Worker Input Message Schemas
// ============================================================================

export const SetItemsMessageSchema = z.object({
    type: z.literal('SET_ITEMS'),
    payload: z.array(LayoutItemInputSchema)
});

export const ConfigureMessageSchema = z.object({
    type: z.literal('CONFIGURE'),
    payload: LayoutConfigSchema
});

// High-frequency messages (SCROLL, RESIZE) are validated with lightweight type guards
// inside the worker to prevent Zod parsing overhead on 60fps events.

// ============================================================================
// Worker Output Message Schemas
// ============================================================================

const ItemPositionSchema = z.object({
    id: z.number().int().positive(),
    x: z.number(),
    y: z.number(),
    width: z.number().nonnegative(),
    height: z.number().nonnegative()
});

export const LayoutCompleteMessageSchema = z.object({
    type: z.literal('LAYOUT_COMPLETE'),
    payload: z.object({
        totalHeight: z.number().nonnegative()
    })
});

export const ErrorMessageSchema = z.object({
    type: z.literal('ERROR'),
    payload: z.object({
        message: z.string()
    })
});

export const PositionResultMessageSchema = z.object({
    type: z.literal('POSITION_RESULT'),
    payload: z.object({
        requestId: z.string(),
        position: ItemPositionSchema.nullable()
    })
});
