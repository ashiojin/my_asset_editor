import { ApiPreviewerState } from "../previewer/api"

export type PreviewerState = ApiPreviewerState & { status: string | null }
const null_previewer_state = {
    status: null,
    gltf_dump: null,
    gltf_info: null,
    gltf_path: null,
    gltf_scene_select: null,
    gltf_sorted_scene_names: null,
    scene_info: null,
}
export function nullPreviewerState(): PreviewerState { return { ...null_previewer_state } }
