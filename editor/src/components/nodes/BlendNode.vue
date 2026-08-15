<script setup lang="ts">
import { Position, Handle } from '@vue-flow/core'
import type { NodeProps } from '@vue-flow/core'
import type { BlendNodeProps, BlendNodeData, BlendNodeEvents } from './NodeTypes.ts'

const props = defineProps<NodeProps<BlendNodeData, BlendNodeEvents, 'blend'>>()

</script>

<template>
    <div class="vue-flow__node-default">
        <div>Blend</div>

        <div>{{ data.weight }}</div>

        <Handle type="target" :position="Position.Top"/>
        <Handle type="source" :position="Position.Bottom"/>
    </div>
</template>
