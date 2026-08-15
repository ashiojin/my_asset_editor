<script setup lang="ts">
import { Position, Handle } from '@vue-flow/core'
import type { ClipNodeProps } from './NodeTypes.ts'

const props = defineProps<ClipNodeProps>()

</script>

<template>
    <div class="vue-flow__node-output">
        <div>{{ data.clip_name }}</div>

        <div>{{ data.weight }}</div>

        <Handle type="source" :position="Position.Bottom"/>
    </div>
</template>
