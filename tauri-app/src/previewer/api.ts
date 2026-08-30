import { useVueFlow } from '@vue-flow/core'
import { AdditiveBlendNodeData, BlendNodeData, ClipNodeData, MyGraphNode, RootNodeData } from '../components/nodes/NodeTypes'
import { GraphCommand, GraphCommandType } from '../components/Command'
import { useMaskGroupListStore } from '../stores/MaskGroupList'
export type ApiGraph = {
    nodes: ApiNodes,
    edges: ApiEdges,
    mask_groups: ApiMaskGroups,
}

export type ApiNodes = Record<string, ApiNode>

export type ApiNode = ApiClipNode | ApiBlendNode | ApiAdditiveBlendNode | ApiRootNode
export type ApiClipNode = {
    "Clip": {
        clip: string,
        weight: number,
        mask: ApiMaskId[],
    }
}
export type ApiBlendNode = {
    "Blend": {
        weight: number,
        mask: ApiMaskId[],
    }
}
export type ApiAdditiveBlendNode = {
    "AdditiveBlend": {
        weight: number,
        mask: ApiMaskId[],
    }
}
export type ApiRootNode = "Root"

export type ApiMaskId = number

export type ApiEdges = ApiEdge[]
export type ApiEdge = {
    src: string,
    dest: string,
}

export type ApiMaskGroups = ApiMaskGroup[]
export type ApiMaskGroup = {
    name: string,
    targets: string[],
}

export function getCurrentGraph(): ApiGraph {
    const { getNodes, getEdges } = useVueFlow('animation_graph')
    const mask_group_store = useMaskGroupListStore()

    const mask_group_id_to_idx = new Map<number, number>()
    mask_group_store.mask_group_list.forEach((item, idx) => {
        mask_group_id_to_idx.set(item.id, idx)
    })

    function mask_ids_to_api_idxes(masks: number[]) {
        const api_idxes: ApiMaskId[] = []
        for (let mask_id of masks) {
            const api_idx = mask_group_id_to_idx.get(mask_id)
            if (api_idx === undefined) {
                console.error('getCurrentGraph', 'mask_id not found in mask_group_list', mask_id)
                continue
            }
            api_idxes.push(api_idx)
        }
        return api_idxes
    }

    let api_masks: ApiMaskGroups = mask_group_store.mask_group_list
        .map((item) => {
            return { name: item.name, targets: item.targets }
        })

    let api_nodes: ApiNodes = {}
    let api_edges: ApiEdges = []

    let map_id_to_label_id: Record<string, string> = {}
    for (let node of getNodes.value as MyGraphNode[]) {
        switch (node.type) {
            case "clip":
                const clip_data = node.data as ClipNodeData
                const mask = mask_ids_to_api_idxes(clip_data.masks)
                api_nodes[clip_data.label_id] = {
                    "Clip": {
                        clip: clip_data.clip_name,
                        weight: clip_data.weight,
                        mask,
                    }
                }
                map_id_to_label_id[node.id] = clip_data.label_id
                break
            case "blend":
                const blend_data = node.data as BlendNodeData
                api_nodes[blend_data.label_id] = {
                    "Blend": {
                        weight: blend_data.weight,
                        mask: [],
                    }
                }
                map_id_to_label_id[node.id] = blend_data.label_id
                break
            case "additive-blend":
                const additive_blend_data = node.data as AdditiveBlendNodeData
                api_nodes[additive_blend_data.label_id] = {
                    "AdditiveBlend": {
                        weight: additive_blend_data.weight,
                        mask: [],
                    }
                }
                map_id_to_label_id[node.id] = additive_blend_data.label_id
                break
            case "root":
                const root_data = node.data as RootNodeData
                api_nodes[root_data.label_id] = "Root"
                map_id_to_label_id[node.id] = root_data.label_id
                break
            default:
                console.error('getCurrentGraph', 'wrong node.type', node)
                break
        }
    }

    for (let edge of getEdges.value) {
        api_edges.push({
            src: map_id_to_label_id[edge.source],
            dest: map_id_to_label_id[edge.target],
        })
    }

    return {
        nodes: api_nodes,
        edges: api_edges,
        mask_groups: api_masks,
    }
}


// ================================================================
// GRAPH COMMAND
// ================================================================
export type ApiGraphCommand = ApiPlayRepeatCommand | ApiStopPlayCommand | ApiSetWeightCommand

type ApiPlayRepeatCommand = {
    'PlayRepeat': string, // node name

}
type ApiStopPlayCommand = {
    'StopPlay': string, // node name
}
type ApiSetWeightCommand = {
    'SetWeight': [
        string, // node name
        Number, // weight
    ]
}

export function convertToApiGraphCommand(command: GraphCommand): ApiGraphCommand {
    switch (command.type) {
        case GraphCommandType.PlayRepeat:
            return { 'PlayRepeat': command.selected }
        case GraphCommandType.Stop:
            return { 'StopPlay': command.selected }
        case GraphCommandType.Weight:
            if (command.weight > 1.0 || command.weight < 0.0) {
                throw Error(`Illigal weight: ${command.weight}`)
            }
            return { 'SetWeight': [command.selected, new Number(command.weight)] }
    }
}
