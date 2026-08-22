<script setup lang="ts">
import { ref } from "vue"
import { Connection, Edge, VueFlow, useVueFlow } from '@vue-flow/core'

import type { MyNode } from './nodes/NodeTypes.ts'
import ClipNode from './nodes/ClipNode.vue'
import BlendNode from './nodes/BlendNode.vue'
import AdditiveBlendNode from './nodes/AdditiveBlendNode.vue'
import RootNode from './nodes/RootNode.vue'

import useDragAndDrop from '../composables/useDnD.js'

const { onDragOver, onDrop, onDragLeave } = useDragAndDrop()

const { addEdges } = useVueFlow('animation_graph')

const nodes = ref<MyNode[]>([
    {
        id: 'root',
        type: 'root',
        position:{ x: 50, y:50 },
        data: { label_id: 'root' }
    },

])

const edges = ref<Edge[]>([])

// function onConnectStart({ nodeId, handleType }) {
//     console.log('On connect start', { nodeId, handleType })
// }
// function onConnectEnd(e) {
//     console.log('On connect end', e)
// }
function onConnect(params: Connection) {
    //console.log('On connect', params)
    addEdges(params)
}


</script>

<template>
    <div class="graph_area" @drop="onDrop">
        <VueFlow :nodes="nodes" :edges="edges"
            @connect="onConnect"
            @dragover="onDragOver"
            @dragleave="onDragLeave"
        >
            <template #node-clip="clipNodeProps">
                <ClipNode v-bind="clipNodeProps"/>
            </template>
            <template #node-blend="blendNodeProps">
                <BlendNode v-bind="blendNodeProps"/>
            </template>
            <template #node-additive-blend="additiveBlendNodeProps">
                <AdditiveBlendNode v-bind="additiveBlendNodeProps"/>
            </template>
            <template #node-root="rootNodeProps">
                <RootNode v-bind="rootNodeProps"/>
            </template>
        </VueFlow>
    </div>
</template>

<style scoped>
</style>
<style>
@import '@vue-flow/core/dist/style.css';
@import '@vue-flow/core/dist/theme-default.css';
div.graph_area {
    background: #F0F0F0;
    width: 600px;
    height: 400px;
}
</style>
