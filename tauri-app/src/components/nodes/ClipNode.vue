<script setup lang="ts">
import { Position, Handle } from '@vue-flow/core'
import type { ClipNodeProps } from './NodeTypes.ts'
import WeightSlider from './parts/WeightSlider.vue';
import MaskGroupSelector from './parts/MaskGroupSelector.vue';
import { usePreviewerState } from '../../stores/PreviewState.ts';

const props = defineProps<ClipNodeProps>()

const previewerState = usePreviewerState()

</script>

<template>
    <div class="vue-flow__node-output">
        <div>Clip: {{ data.label_id }}</div>
        <select v-model="data.clip_name">
            <option v-for="clip in previewerState.clip_list" :key="clip" :value="clip">{{ clip }}</option>
        </select>

        <WeightSlider v-model="data.weight"></WeightSlider>

        <MaskGroupSelector v-model="data.masks"></MaskGroupSelector>

        <Handle type="source" :position="Position.Bottom"/>
    </div>
</template>
