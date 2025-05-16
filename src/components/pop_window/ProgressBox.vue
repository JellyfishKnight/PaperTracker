<!-- components/ProgressBox.vue -->
<template>
    <teleport to="body">
      <modal :show="show" :title="title" @close="cancelOperation">
        <div class="progress-content">
          <p class="progress-message">{{ message }}</p>
          
          <div class="progress-bar">
            <div class="progress-bar-fill" :style="{ width: `${progress}%` }"></div>
          </div>
          
          <p class="progress-percent">{{ progress }}%</p>
        </div>
        <template #footer>
          <button 
            v-if="cancelable"
            @click="cancelOperation"
            :disabled="progress >= 100"
          >
            取消
          </button>
          <button 
            v-if="progress >= 100"
            @click="closeModal"
          >
            完成
          </button>
        </template>
      </modal>
    </teleport>
  </template>
  
  <script setup lang="ts">
  import { ref, defineExpose } from 'vue';
  import Modal from './Modal.vue';
  
  interface ProgressConfig {
    message?: string;
    title?: string;
    initialProgress?: number;
    cancelable?: boolean;
    onComplete?: () => void;
    onCancel?: () => void;
    autoProgress?: (currentProgress: number) => number;
  }
  
  interface ProgressControl {
    updateProgress: (newProgress: number) => void;
    complete: (finalMessage?: string) => void;
    cancel: () => void;
  }
  
  const show = ref<boolean>(false);
  const title = ref<string>('操作进行中');
  const message = ref<string>('');
  const progress = ref<number>(0);
  const cancelable = ref<boolean>(true);
  const onComplete = ref<(() => void) | null>(null);
  const onCancel = ref<(() => void) | null>(null);
  let operationId: number | null = null;
  
  function openProgress(config: ProgressConfig): ProgressControl {
    message.value = config.message || '正在执行操作...';
    title.value = config.title || '操作进行中';
    progress.value = config.initialProgress || 0;
    cancelable.value = config.cancelable !== false;
    onComplete.value = config.onComplete || null;
    onCancel.value = config.onCancel || null;
    show.value = true;
    
    // 清除任何现有的操作ID
    if (operationId) {
      clearInterval(operationId);
    }
    
    // 如果提供了自动进度更新函数，则使用它
    if (typeof config.autoProgress === 'function') {
      operationId = window.setInterval(() => {
        const newProgress = config.autoProgress!(progress.value);
        updateProgress(newProgress);
        
        if (newProgress >= 100) {
          if (operationId) {
            clearInterval(operationId);
            operationId = null;
          }
        }
      }, 100);
    }
    
    return {
      // 返回更新进度的函数，以便调用者可以手动更新进度
      updateProgress,
      // 返回一个完成函数，以便调用者可以手动标记操作为完成
      complete(finalMessage?: string) {
        if (finalMessage) {
          message.value = finalMessage;
        }
        progress.value = 100;
        if (operationId) {
          clearInterval(operationId);
          operationId = null;
        }
        if (typeof onComplete.value === 'function') {
          onComplete.value();
        }
      },
      // 返回一个取消函数，以便调用者可以手动取消操作
      cancel() {
        cancelOperation();
      }
    };
  }
  
  function updateProgress(newProgress: number): void {
    progress.value = Math.min(100, Math.max(0, newProgress));
  }
  
  function cancelOperation(): void {
    if (progress.value >= 100) {
      closeModal();
      return;
    }
    
    if (operationId) {
      clearInterval(operationId);
      operationId = null;
    }
    
    show.value = false;
    if (typeof onCancel.value === 'function') {
      onCancel.value();
    }
  }
  
  function closeModal(): void {
    show.value = false;
    if (typeof onComplete.value === 'function') {
      onComplete.value();
    }
  }
  
  defineExpose({
    open: openProgress,
    close: closeModal,
    updateProgress
  });
  </script>
  
  <style scoped>
  .progress-content {
    display: flex;
    flex-direction: column;
    gap: 15px;
    min-width: 350px;
  }
  
  .progress-message {
    margin: 0;
    white-space: pre-line;
  }
  
  .progress-bar {
    width: 100%;
    height: 25px;
    background-color: var(--widget-background);
    border: 1px solid var(--border-color);
    border-radius: 10px;
    overflow: hidden;
  }
  
  .progress-bar-fill {
    height: 100%;
    background-color: var(--highlight-color);
    border-radius: 5px;
    transition: width 0.3s ease;
  }
  
  .progress-percent {
    text-align: center;
    font-weight: bold;
    margin: 0;
  }
  </style>