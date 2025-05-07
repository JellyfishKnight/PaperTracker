<template>
  <div class="app-container">
    <main-window v-if="currentWindow === 'main'" @open-face-tracker="openFaceTracker" @open-eye-tracker="openEyeTracker" />
    <face-tracker-window v-if="currentWindow === 'face'" @back-to-main="openMainWindow" />
    <eye-tracker-window v-if="currentWindow === 'eye'" @back-to-main="openMainWindow" />
  </div>
</template>

<script setup>
import { ref } from 'vue';
import MainWindow from './components/MainWindow.vue';
import FaceTrackerWindow from './components/FaceTrackerWindow.vue';
import EyeTrackerWindow from './components/EyeTrackerWindow.vue';

const currentWindow = ref('main');

function openMainWindow() {
  currentWindow.value = 'main';
}

function openFaceTracker() {
  currentWindow.value = 'face';
}

function openEyeTracker() {
  currentWindow.value = 'eye';
}
</script>

<style>
:root {
  --background-color: #2F343F;
  --widget-background: #1e1d23;
  --text-color: #a9b7c6;
  --highlight-color: #04b97f;
  --highlight-hover: #37efba;
  --disabled-color: #808086;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  color: var(--text-color);
  background-color: var(--background-color);
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.app-container {
  width: 100%;
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-start;
  padding: 20px;
}

button {
  border-style: solid;
  border-color: transparent;
  border-bottom-color: transparent;
  border-width: 1px;
  color: var(--text-color);
  padding: 8px 12px;
  background-color: var(--widget-background);
  cursor: pointer;
  transition: all 0.3s ease;
  font-size: 1em;
}

button:hover {
  border-bottom-color: var(--highlight-hover);
  color: #FFFFFF;
}

button:active {
  border-bottom-color: var(--highlight-hover);
  border-bottom-width: 2px;
  color: var(--highlight-hover);
}

button:disabled {
  border-bottom-color: var(--disabled-color);
  color: var(--disabled-color);
  cursor: not-allowed;
}

input, textarea {
  background-color: var(--widget-background);
  color: var(--text-color);
  border: 1px solid #3a3a3a;
  border-radius: 4px;
  padding: 8px;
}

.progress-bar {
  width: 100%;
  height: 20px;
  background-color: var(--widget-background);
  border: 1px solid #3a3a3a;
  border-radius: 10px;
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  background-color: var(--highlight-color);
  border-radius: 5px;
  transition: width 0.3s ease;
}

.slider {
  width: 100%;
  height: 5px;
  background: white;
  position: relative;
  border-radius: 2px;
  cursor: pointer;
}

.slider .track {
  height: 100%;
  background: var(--highlight-color);
  width: 50%;
  border-radius: 2px;
}

.slider .thumb {
  width: 14px;
  height: 14px;
  background: #b4b4b4;
  border: 1px solid #5c5c5c;
  border-radius: 50%;
  position: absolute;
  top: 50%;
  transform: translate(-50%, -50%);
  left: 50%;
  cursor: grab;
}

.slider .thumb:active {
  cursor: grabbing;
}

.status-label {
  font-style: italic;
  font-weight: bold;
}

.stacked-widget {
  width: 100%;
  max-width: 900px;
}

.no-image {
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #333;
  color: #aaa;
  text-align: center;
}

/* Scrollbar styling */
::-webkit-scrollbar {
  width: 14px;
  height: 14px;
  background-color: rgb(11, 54, 117);
}

::-webkit-scrollbar-thumb {
  background-color: var(--text-color);
  border-radius: 7px;
}

::-webkit-scrollbar-thumb:hover {
  background-color: #cbd4dd;
}

::-webkit-scrollbar-track {
  background-color: var(--widget-background);
}
</style>