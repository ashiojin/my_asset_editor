import { useVueFlow } from '@vue-flow/core'
import { AdditiveBlendNodeData, BlendNodeData, ClipNodeData, MyGraphNode, NodeData, NodeEvents, RootNodeData } from './components/nodes/NodeTypes'
export type ApiGraph = {
    nodes: ApiNodes,
    edges: ApiEdges,
    mask_groups: ApiMaskGroups,
}

export type ApiNodes = Record<string, ApiNode>

export type ApiNode = ApiClipNode | ApiBlendNode | ApiAdditiveBlendNode | ApiRootNode
export type ApiClipNode = { "Clip": {
    clip: string,
    weight: number,
    mask: ApiMaskId[],
}}
export type ApiBlendNode = { "Blend": {
    weight: number,
    mask: ApiMaskId[],
}}
export type ApiAdditiveBlendNode = { "AdditiveBlend": {
    weight: number,
    mask: ApiMaskId[],
}}
export type ApiRootNode = "Root"

export type ApiMaskId = number

export type ApiEdges = ApiEdge[]
export type ApiEdge = {
    src: string,
    dest: string,
}

export type ApiMaskGroups = ApiMaskGroup[]
export type ApiMaskGroup = {
    targets: string[],
}

export function getCurrentGraph(): ApiGraph {
    const { getNodes, getEdges, getNodeTypes } = useVueFlow('animation_graph')

    let api_masks: ApiMaskGroups = [] // TODO: imprements

    let api_nodes: ApiNodes = {}
    let api_edges: ApiEdges = []

    console.log('getNodeTypes', getNodeTypes.value)
    console.log('getNodes', getNodes.value)
    let map_id_to_label_id: Record<string, string> = {}
    for (let node of getNodes.value as MyGraphNode[]) {
        switch (node.type) {
            case "clip":
                const clip_data = node.data as ClipNodeData
                api_nodes[clip_data.label_id] = {"Clip":{
                    clip: clip_data.clip_name,
                    weight: clip_data.weight,
                    mask: [],
                }}
                map_id_to_label_id[node.id] = clip_data.label_id
            break
            case "blend":
                const blend_data = node.data as BlendNodeData
                api_nodes[blend_data.label_id] = {"Blend":{
                    weight: blend_data.weight,
                    mask: [],
                }}
                map_id_to_label_id[node.id] = blend_data.label_id
            break
            case "additive-blend":
                const additive_blend_data = node.data as AdditiveBlendNodeData
                api_nodes[additive_blend_data.label_id] = {"AdditiveBlend": {
                    weight: additive_blend_data.weight,
                    mask: [],
                }}
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
