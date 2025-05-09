<!-- App.vue -->
<template>
  <AppLayout v-model:activePage="activePage">
    <MainWindow v-show="activePage === 'main'" />
    <EyeTrackerWindow v-show="activePage === 'eye'" />
    <FaceTrackerWindow v-show="activePage === 'face'" />
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