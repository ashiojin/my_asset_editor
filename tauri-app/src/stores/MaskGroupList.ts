import { defineStore } from "pinia";
import { ref } from "vue";

export interface MaskGroup {
    /// the id of the mask group, used to identify the group in the graph
    id: number,
    /// name can be any string and be changed after definition, it is used to display the name of the group in the graph
    name: string,
    /// the list of the target names. target names are the names of the bones in the gltf file.
    targets: string[],
}
export const useMaskGroupListStore = defineStore('maskGroupList', () => {

    const mask_group_list = ref<MaskGroup[]>([])

    return {
        mask_group_list,
    }
})
