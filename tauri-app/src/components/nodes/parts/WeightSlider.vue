<script setup lang="ts">
const ratio = defineModel<number>({ required: true })

</script>

<template>
    <div class="ratio-slider">
        <div class="slider-header">
            <label>Weight:</label>
            <input
                class="nodrag"
                type="number"
                v-model.number="ratio"
                min="0.0"
                max="1.0"
                step="0.01"
            />
        </div>
        <input
            class="nodrag"
            type="range"
            v-model.number="ratio"
            min="0.0"
            max="1.0"
            step="0.01"
        />
    </div>
</template>

<style scoped>
.ratio-slider {
    display: flex;
    flex-direction: column;
    gap: 4px;
    background: #fcfcfc;
    border: 1px solid #ddd;
    padding: 6px;
    border-radius: 4px;
}
.slider-header {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
    color: #666;
}
.value-display {
    font-weight: bold;
    color: #333;
}
input[type="range"] {
    width: 100%;
    cursor: pointer;
}
</style>
