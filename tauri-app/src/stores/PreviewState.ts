import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { nullPreviewerState, PreviewerState } from "../components/ToolTypes";

export const usePreviewerState = defineStore('PreviewerState', () => {

    const state = ref<PreviewerState>(nullPreviewerState())

    const clip_list = computed(() => {
        return state.value.gltf_info?.animations.map(a => a.name) ?? []
    })

    return {
        // ref
        state,
        // computed
        clip_list,
    }
})
