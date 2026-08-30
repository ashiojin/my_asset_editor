// The globals for the graph
//
// - The list of the mask groups that can be defined by the user for the graph
//
//
import type { InjectionKey, ComputedRef } from "vue";



/// Mask group that can be defined by the user for the graph.
/// CRUD operations can be performed on the mask groups.
export interface MaskGroup {
    /// the id of the mask group, used to identify the group in the graph
    id: number,
    /// name can be any string and be changed after definition, it is used to display the name of the group in the graph
    name: string,
    /// the list of the target names. target names are the names of the bones in the gltf file.
    targets: string[],
}


export const MaskGroupListKey: InjectionKey<ComputedRef<MaskGroup[]>> = Symbol('MaskGroupListKey')


