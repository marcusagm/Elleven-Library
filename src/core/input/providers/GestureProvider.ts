/**
 * Gesture Provider
 * Recognizes multi-touch gestures (swipe, pinch, rotate)
 */

import { createGestureToken } from '../normalizer';
import { dispatchToken } from '../dispatcher';
import { inputStore } from '../store/inputStore';
import type { GesturePayload, SwipeDirection } from '../types';

interface GestureConfig {
    throttleMs: number;
    minSwipeDistance: number;
}

const DEFAULT_CONFIG: GestureConfig = {
    throttleMs: 50,
    minSwipeDistance: 30
};

let attached = false;
let globalConfig = { ...DEFAULT_CONFIG };
let lastGestureTimestamp = 0;

interface TouchPoint {
    id: number;
    x: number;
    y: number;
}

interface TouchTracking {
    startTime: number;
    startTouches: TouchPoint[];
    lastTouches: TouchPoint[];
    moved: boolean;
}

interface PinchTracking {
    startDist: number;
    lastDist: number;
}

interface RotateTracking {
    startAngle: number;
    lastAngle: number;
}

let currentTouchTracking: TouchTracking | null = null;
let currentPinchTracking: PinchTracking | null = null;
let currentRotateTracking: RotateTracking | null = null;

type GestureCallback = (payload: GesturePayload) => void;
let globalGestureCallback: GestureCallback | null = null;

function shouldDispatch(): boolean {
    if (globalConfig.throttleMs <= 0) return true;

    const currentTimestamp = Date.now();
    if (currentTimestamp - lastGestureTimestamp >= globalConfig.throttleMs) {
        lastGestureTimestamp = currentTimestamp;
        return true;
    }
    return false;
}

function computeAveragePoint(points: TouchPoint[]): { x: number; y: number } {
    let sumX = 0;
    let sumY = 0;
    for (const point of points) {
        sumX += point.x;
        sumY += point.y;
    }
    return { x: sumX / points.length, y: sumY / points.length };
}

function computeDistance(point1: TouchPoint, point2: TouchPoint): number {
    return Math.hypot(point2.x - point1.x, point2.y - point1.y);
}

function computeAngle(point1: TouchPoint, point2: TouchPoint): number {
    return Math.atan2(point2.y - point1.y, point2.x - point1.x);
}

function emitGesture(payload: GesturePayload): void {
    if (globalGestureCallback) {
        globalGestureCallback(payload);
    }

    const token = createGestureToken(payload.gesture, payload.meta as Record<string, unknown>);
    dispatchToken(token, payload.event);
}

function handlePinch(touches: TouchPoint[], event: TouchEvent): void {
    if (!currentPinchTracking || !shouldDispatch()) return;

    const distance = computeDistance(touches[0], touches[1]);
    const scale = distance / (currentPinchTracking.startDist || 1);
    currentPinchTracking.lastDist = distance;

    const center = {
        x: (touches[0].x + touches[1].x) / 2,
        y: (touches[0].y + touches[1].y) / 2
    };

    emitGesture({
        gesture: 'pinch',
        meta: { scale, center, incremental: true },
        event
    });
}

function handleRotate(touches: TouchPoint[], event: TouchEvent): void {
    if (!currentRotateTracking || !shouldDispatch()) return;

    const angle = computeAngle(touches[0], touches[1]);
    const deltaRadians = angle - currentRotateTracking.startAngle;
    const deltaDegrees = deltaRadians * (180 / Math.PI);
    currentRotateTracking.lastAngle = angle;

    const center = {
        x: (touches[0].x + touches[1].x) / 2,
        y: (touches[0].y + touches[1].y) / 2
    };

    emitGesture({
        gesture: 'rotate',
        meta: { angle: deltaDegrees, center, incremental: true },
        event
    });
}

function onTouchStart(event: TouchEvent): void {
    if (!inputStore.enabled()) return;
    if (!event.touches || event.touches.length === 0) return;

    const currentTimestamp = Date.now();
    const touches: TouchPoint[] = Array.from(event.touches).map(touch => ({
        id: touch.identifier,
        x: touch.clientX,
        y: touch.clientY
    }));

    currentTouchTracking = {
        startTime: currentTimestamp,
        startTouches: touches,
        lastTouches: [...touches],
        moved: false
    };

    if (touches.length === 2) {
        const distance = computeDistance(touches[0], touches[1]);
        const angle = computeAngle(touches[0], touches[1]);
        currentPinchTracking = { startDist: distance, lastDist: distance };
        currentRotateTracking = { startAngle: angle, lastAngle: angle };
    } else {
        currentPinchTracking = null;
        currentRotateTracking = null;
    }
}

function onTouchMove(event: TouchEvent): void {
    if (!inputStore.enabled()) return;
    if (!currentTouchTracking || !event.touches) return;

    const touches: TouchPoint[] = Array.from(event.touches).map(touch => ({
        id: touch.identifier,
        x: touch.clientX,
        y: touch.clientY
    }));

    currentTouchTracking.lastTouches = touches;
    currentTouchTracking.moved = true;

    if (touches.length === 2) {
        handlePinch(touches, event);
        handleRotate(touches, event);
    }
}

function detectSwipe(data: TouchTracking, event: TouchEvent): boolean {
    const startAverage = computeAveragePoint(data.startTouches);
    const endAverage = computeAveragePoint(data.lastTouches);
    const deltaX = endAverage.x - startAverage.x;
    const deltaY = endAverage.y - startAverage.y;
    const absoluteDeltaX = Math.abs(deltaX);
    const absoluteDeltaY = Math.abs(deltaY);

    if (
        data.moved &&
        (absoluteDeltaX >= globalConfig.minSwipeDistance ||
            absoluteDeltaY >= globalConfig.minSwipeDistance)
    ) {
        let direction: SwipeDirection;
        if (absoluteDeltaX >= absoluteDeltaY) {
            direction = deltaX > 0 ? 'right' : 'left';
        } else {
            direction = deltaY > 0 ? 'down' : 'up';
        }

        emitGesture({
            gesture: 'swipe',
            meta: {
                fingers: Math.min(data.startTouches.length, data.lastTouches.length),
                direction,
                deltaX,
                deltaY,
                duration: Date.now() - data.startTime,
                final: true
            },
            event
        });
        return true;
    }
    return false;
}

function finalizeGestures(event: TouchEvent): void {
    if (currentPinchTracking && currentPinchTracking.startDist) {
        const finalScale = currentPinchTracking.lastDist / currentPinchTracking.startDist;
        emitGesture({
            gesture: 'pinch',
            meta: { scale: finalScale, final: true },
            event
        });
    }

    if (currentRotateTracking && typeof currentRotateTracking.lastAngle === 'number') {
        const deltaDegrees =
            (currentRotateTracking.lastAngle - currentRotateTracking.startAngle) * (180 / Math.PI);
        emitGesture({
            gesture: 'rotate',
            meta: { angle: deltaDegrees, final: true },
            event
        });
    }
}

function onTouchEnd(event: TouchEvent): void {
    if (!inputStore.enabled()) return;
    if (!currentTouchTracking) return;

    const fingers = Math.min(
        currentTouchTracking.startTouches.length,
        currentTouchTracking.lastTouches.length
    );

    if (fingers > 0) {
        detectSwipe(currentTouchTracking, event);
        finalizeGestures(event);
    }

    currentTouchTracking = null;
    currentPinchTracking = null;
    currentRotateTracking = null;
}

export interface GestureProviderOptions {
    throttleMs?: number;
    minSwipeDistance?: number;
    onGesture?: GestureCallback;
}

export function createGestureProvider(options?: GestureProviderOptions): () => void {
    if (attached) {
        console.warn('[GestureProvider] Already attached');
        return () => {};
    }

    if (typeof window === 'undefined') return () => {};

    if (options?.throttleMs !== undefined) globalConfig.throttleMs = options.throttleMs;
    if (options?.minSwipeDistance !== undefined)
        globalConfig.minSwipeDistance = options.minSwipeDistance;
    if (options?.onGesture) globalGestureCallback = options.onGesture;

    window.addEventListener('touchstart', onTouchStart, { passive: true });
    window.addEventListener('touchmove', onTouchMove, { passive: false });
    window.addEventListener('touchend', onTouchEnd, { passive: true });

    attached = true;

    return () => {
        window.removeEventListener('touchstart', onTouchStart);
        window.removeEventListener('touchmove', onTouchMove);
        window.removeEventListener('touchend', onTouchEnd);

        currentTouchTracking = null;
        currentPinchTracking = null;
        currentRotateTracking = null;
        globalGestureCallback = null;
        globalConfig = { ...DEFAULT_CONFIG };
        attached = false;
    };
}

export function isGestureProviderAttached(): boolean {
    return attached;
}
