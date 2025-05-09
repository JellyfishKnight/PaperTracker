<!-- components/Modal.vue -->
<template>
    <transition name="modal-fade">
      <div v-if="show" class="modal-backdrop" @click="$emit('close')">
        <div class="modal-container" @click.stop>
          <div class="modal-header">
            <h3>{{ title }}</h3>
            <button class="close-button" @click="$emit('close')">&times;</button>
          </div>
          <div class="modal-body">
            <slot></slot>
          </div>
          <div class="modal-footer">
            <slot name="footer">
              <button @click="$emit('close')">确定</button>
            </slot>
          </div>
        </div>
      </div>
    </transition>
  </template>
  
  <script setup>
  defineProps({
    show: Boolean,
    title: {
      type: String,
      default: '提示'
    }
  });
  
  defineEmits(['close']);
  </script>
  
  <style scoped>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
  }
  
  .modal-container {
    width: 400px;
    max-width: 90%;
    background-color: var(--background-color);
    border-radius: 8px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  
  .modal-header {
    padding: 15px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--border-color);
  }
  
  .modal-header h3 {
    margin: 0;
    color: var(--text-color);
  }
  
  .close-button {
    background: none;
    border: none;
    font-size: 24px;
    cursor: pointer;
    color: var(--text-color);
    padding: 0 5px;
  }
  
  .modal-body {
    padding: 20px;
    overflow-y: auto;
    max-height: 60vh;
  }
  
  .modal-footer {
    padding: 15px;
    display: flex;
    justify-content: flex-end;
    border-top: 1px solid var(--border-color);
  }
  
  .modal-footer button {
    margin-left: 10px;
  }
  
  /* Transition animations */
  .modal-fade-enter-active,
  .modal-fade-leave-active {
    transition: opacity 0.3s;
  }
  
  .modal-fade-enter-from,
  .modal-fade-leave-to {
    opacity: 0;
  }
  </style>