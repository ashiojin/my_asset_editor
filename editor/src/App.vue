<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

import Graph from './components/Graph.vue'
import Makeshift from './components/Makeshift.vue'

import useDropAndDrag from './composables/useDnD.js'


</script>

<template>
  <main class="container">
    <h1>Animation Graph</h1>

    <div class="row">
        <Graph></Graph>
    </div>

    <div class="row">
        <Makeshift></Makeshift>
    </div>
  </main>
</template>

<style scoped>
</style>
<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 0vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.row {
  display: flex;
  justify-content: center;
}

</style>
