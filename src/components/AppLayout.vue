<!-- AppLayout.vue -->
<template>
  <div class="app-container">
    <!-- 左侧导航栏 -->
    <div class="nav-sidebar">
      <button 
        :class="{ 'nav-active': activePage === 'main' }"
        @click="changePage('main')"
      >
        主页面
      </button>
      <button 
        :class="{ 'nav-active': activePage === 'eye' }"
        @click="changePage('eye')"
      >
        眼追界面
      </button>
      <button 
        :class="{ 'nav-active': activePage === 'face' }"
        @click="changePage('face')"
      >
        面捕界面
      </button>
    </div>

    <!-- 主内容区域 -->
    <div class="content-area">
      <slot></slot>
    </div>
  </div>
</template>

<script setup>
import { defineProps, defineEmits } from 'vue';

const props = defineProps({
  activePage: String
});

const emit = defineEmits(['update:activePage']);

function changePage(page) {
  emit('update:activePage', page);
}
</script>

<style scoped>
.app-container {
  display: flex;
  height: 100vh;
  width: 100%;
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  color: var(--text-color);
}

.nav-sidebar {
  width: 100px;
  background-color: var(--widget-background);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  padding: 10px 0;
}

.nav-sidebar button {
  height: 80px;
  margin: 5px;
  border-width: 1px;
  border-style: solid;
  border-color: transparent;
  border-left-width: 3px;
  transition: all 0.3s ease;
}

.nav-sidebar button:hover {
  border-bottom-color: var(--highlight-hover);
  color: #FFFFFF;
}

.nav-sidebar button.nav-active {
  border-left-color: var(--highlight-color);
  background-color: rgba(4, 185, 127, 0.1);
  color: var(--highlight-color);
}

.content-area {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
}
</style>