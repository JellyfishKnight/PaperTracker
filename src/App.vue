<!-- App.vue -->
<template>
  <AppLayout v-model:activePage="activePage">
    <MainWindow v-if="activePage === 'main'" />
    <EyeTrackerWindow v-if="activePage === 'eye'" />
    <FaceTrackerWindow v-if="activePage === 'face'" />
  </AppLayout>
</template>

<script setup>
import { ref } from 'vue';
import AppLayout from './components/AppLayout.vue';
import MainWindow from './components/MainWindow.vue';
import EyeTrackerWindow from './components/EyeTrackerWindow.vue';
import FaceTrackerWindow from './components/FaceTrackerWindow.vue';

const activePage = ref('main');
</script>

<style>
/* 可以在这里添加全局样式 */
</style>