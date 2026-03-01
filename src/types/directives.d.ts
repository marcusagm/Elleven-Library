import { AssetDnDParams } from '../core/dnd/assetDirective';
import { AssetDragSourceParams } from '../core/dnd/assetDragSource';

declare module 'solid-js' {
    namespace JSX {
        interface Directives {
            assetDnD: AssetDnDParams;
            assetDragSource: AssetDragSourceParams;
        }
    }
}
