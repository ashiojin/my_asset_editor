import type { NodeProps, Node, GraphNode } from '@vue-flow/core'

// Root Node
export interface RootNodeData {
    label_id: string,
}
export type RootNodeEvents = {}
export type RootNodeProps = NodeProps<RootNodeData, RootNodeEvents, 'root'>

// Blend Node
export interface BlendNodeData {
    label_id: string,
    weight: number,
}
export type BlendNodeEvents = {}
export type BlendNodeProps = NodeProps<BlendNodeData, BlendNodeEvents, 'blend'>

// Additive Blend Node
export interface AdditiveBlendNodeData {
    label_id: string,
    weight: number,
}
export type AdditiveBlendNodeEvents = {}
export type AdditiveBlendNodeProps = NodeProps<AdditiveBlendNodeData, AdditiveBlendNodeEvents, 'additive-blend'>

// Clip Node
export interface ClipNodeData {
    label_id: string,
    clip_name: string,
    weight: number,
}
export type ClipNodeEvents = {}
export type ClipNodeProps = NodeProps<ClipNodeData, ClipNodeEvents, 'clip'>

//
export type NodeData = AdditiveBlendNodeData | BlendNodeData | ClipNodeData | RootNodeData
export type NodeEvents = AdditiveBlendNodeEvents | BlendNodeEvents | ClipNodeEvents | RootNodeEvents

export type NodeTypeName = 'additive-blend' | 'blend' | 'clip' | 'root'

export type MyNode = Node<AdditiveBlendNodeData, AdditiveBlendNodeEvents, 'additive-blend'> | Node<BlendNodeData, BlendNodeEvents, 'blend'> | Node<ClipNodeData, ClipNodeEvents, 'clip'> | Node<RootNodeData, RootNodeEvents, 'root'>


export type MyGraphNode = GraphNode<AdditiveBlendNodeData, AdditiveBlendNodeEvents, 'additive-blend'> | GraphNode<BlendNodeData, BlendNodeEvents, 'blend'> | GraphNode<ClipNodeData, ClipNodeEvents, 'clip'> | GraphNode<RootNodeData, RootNodeEvents, 'root'>


