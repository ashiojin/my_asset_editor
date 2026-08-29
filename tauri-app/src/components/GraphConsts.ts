import type { DeepReadonly, InjectionKey, Ref } from "vue";

// === Constants From Gltf
// - targets: the list of the bones
// - clips: the list of the actions
export interface MaskTarget {
    /// the target name
    target: string,
    /// path list of the names to the target
    path: string[],
}

export const MaskTargetListKey: InjectionKey<DeepReadonly<Ref<MaskTarget[]>>> = Symbol('MaskTargetListKey')


export interface Clip {
    name: string,
}

export const ClipListKey: InjectionKey<Ref<Clip[]>> = Symbol('ClipListKey')

