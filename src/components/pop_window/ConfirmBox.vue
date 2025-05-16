<!-- components/ConfirmBox.vue -->
<template>
  <teleport to="body">
    <modal :show="show" :title="title" @close="cancel">
      <div class="confirm-content">
        <div class="icon warning-icon" v-if="showIcon">!</div>
        <p>{{ message }}</p>
      </div>
      <template #footer>
        <button class="cancel-button" @click="cancel">{{ cancelText }}</button>
        <button class="confirm-button" @click="confirm">{{ confirmText }}</button>
      </template>
    </modal>
  </teleport>
</template>

<script setup lang="ts">
import { ref, defineExpose } from 'vue';
import Modal from './Modal.vue';

interface ConfirmConfig {
  message: string;
  title?: string;
  confirmText?: string;
  cancelText?: string;
  showIcon?: boolean;
  onConfirm?: () => void;
  onCancel?: () => void;
}

const show = ref<boolean>(false);
const message = ref<string>('');
const title = ref<string>('确认');
const confirmText = ref<string>('确认');
const cancelText = ref<string>('取消');
const showIcon = ref<boolean>(true);
const onConfirm = ref<(() => void) | null>(null);
const onCancel = ref<(() => void) | null>(null);

function openConfirm(config: ConfirmConfig): void {
  message.value = config.message || '';
  title.value = config.title || '确认';
  confirmText.value = config.confirmText || '确认';
  cancelText.value = config.cancelText || '取消';
  showIcon.value = config.showIcon !== false;
  onConfirm.value = config.onConfirm || null;
  onCancel.value = config.onCancel || null;
  show.value = true;
}

function confirm(): void {
  show.value = false;
  if (typeof onConfirm.value === 'function') {
    onConfirm.value();
  }
}

function cancel(): void {
  show.value = false;
  if (typeof onCancel.value === 'function') {
    onCancel.value();
  }
}

defineExpose({
  open: openConfirm,
  close: cancel
});
</script>

<style scoped>
.confirm-content {
  display: flex;
  align-items: flex-start;
  gap: 15px;
}

.icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 50%;
  font-weight: bold;
  font-size: 18px;
  background-color: rgba(255, 186, 0, 0.2);
  color: #ffba00;
}

.cancel-button {
  background-color: var(--widget-background);
}

.confirm-button {
  background-color: var(--highlight-color);
  color: white;
}

.confirm-button:hover {
  background-color: var(--highlight-hover);
}
</style>