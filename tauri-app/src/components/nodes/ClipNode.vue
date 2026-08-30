<script setup lang="ts">
import { Position, Handle } from '@vue-flow/core'
import type { ClipNodeProps } from './NodeTypes.ts'
import WeightSlider from './parts/WeightSlider.vue';
import MaskGroupSelector from './parts/MaskGroupSelector.vue';

const props = defineProps<ClipNodeProps>()

</script>

<template>
    <div class="vue-flow__node-output">
        <div>{{ data.clip_name }}</div>

        <WeightSlider v-model="data.weight"></WeightSlider>

        <MaskGroupSelector v-model="data.masks"></MaskGroupSelector>

        <Handle type="source" :position="Position.Bottom"/>
    </div>
</template>
