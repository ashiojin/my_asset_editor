<script setup lang="ts">
import { Position, Handle } from '@vue-flow/core'
import type { ClipNodeProps } from './NodeTypes.ts'
import WeightSlider from './parts/WeightSlider.vue';

const props = defineProps<ClipNodeProps>()

</script>

<template>
    <div class="vue-flow__node-output">
        <div>{{ data.clip_name }}</div>

        <WeightSlider v-model="data.weight"></WeightSlider>

        <Handle type="source" :position="Position.Bottom"/>
    </div>
</template>
