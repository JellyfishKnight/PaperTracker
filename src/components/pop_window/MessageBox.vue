<!-- components/MessageBox.vue -->
<template>
  <teleport to="body">
    <modal :show="show" :title="title" @close="closeModal">
      <div class="message-content" :class="type">
        <div class="icon" v-if="type !== 'default'">
          <div v-if="type === 'success'" class="success-icon">✓</div>
          <div v-if="type === 'error'" class="error-icon">✕</div>
          <div v-if="type === 'warning'" class="warning-icon">!</div>
          <div v-if="type === 'info'" class="info-icon">i</div>
        </div>
        <p class="message-text">{{ message }}</p>
      </div>
      <template #footer>
        <button @click="closeModal">{{ buttonText }}</button>
      </template>
    </modal>
  </teleport>
</template>

<script setup>
import { ref, defineExpose } from 'vue';
import Modal from './Modal.vue';

const show = ref(false);
const message = ref('');
const title = ref('提示');
const type = ref('default'); // default, success, error, warning, info
const buttonText = ref('确定');
const callback = ref(null);

function openModal(config) {
  message.value = config.message || '';
  title.value = config.title || '提示';
  type.value = config.type || 'default';
  buttonText.value = config.buttonText || '确定';
  callback.value = config.callback || null;
  show.value = true;
}

function closeModal() {
  show.value = false;
  if (typeof callback.value === 'function') {
    callback.value();
  }
}

defineExpose({
  open: openModal,
  close: closeModal
});
</script>

<style scoped>
.message-content {
  display: flex;
  align-items: flex-start;
  gap: 15px;
}

.message-text {
  white-space: pre-line; /* 添加这一行使换行符生效 */
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
}

.success .icon {
  background-color: rgba(4, 185, 127, 0.2);
  color: var(--highlight-color);
}

.error .icon {
  background-color: rgba(255, 76, 76, 0.2);
  color: #ff4c4c;
}

.warning .icon {
  background-color: rgba(255, 186, 0, 0.2);
  color: #ffba00;
}

.info .icon {
  background-color: rgba(79, 139, 255, 0.2);
  color: #4f8bff;
}

.success-icon, .error-icon, .warning-icon, .info-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
}
</style>