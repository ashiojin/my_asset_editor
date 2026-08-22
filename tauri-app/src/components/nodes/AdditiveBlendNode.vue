<script setup lang="ts">
import { Position, Handle } from '@vue-flow/core'
import type { AdditiveBlendNodeProps } from './NodeTypes.ts'

const props = defineProps<AdditiveBlendNodeProps>()

</script>

<template>
    <div class="vue-flow__node-default">
        <div>Additive Blend</div>

        <div>{{ data.weight }}</div>

        <Handle type="target" :position="Position.Top"/>
        <Handle type="source" :position="Position.Bottom"/>
    </div>
</template>
