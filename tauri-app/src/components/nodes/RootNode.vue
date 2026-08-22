<script setup lang="ts">
import { Position, Handle } from '@vue-flow/core'
import type { RootNodeProps } from './NodeTypes.ts'

const props = defineProps<RootNodeProps>()

</script>

<template>
    <div class="vue-flow__node-input">
        <div>Root</div>

        <Handle type="target" :position="Position.Top"/>
    </div>
</template>
