<!-- components/FirmwareSelector.vue -->
<template>
    <teleport to="body">
      <modal :show="show" :title="title" @close="cancelSelection">
        <div class="firmware-selector-content">
          <p class="description">{{ description }}</p>
          
          <div class="selection-container">
            <div class="device-type-selector">
              <h4>设备类型</h4>
              <div class="radio-group">
                <div class="radio-item">
                  <input type="radio" id="device-face" v-model="selectedDevice" value="face" />
                  <label for="device-face">面捕设备</label>
                </div>
                <div class="radio-item">
                  <input type="radio" id="device-left-eye" v-model="selectedDevice" value="left_eye" />
                  <label for="device-left-eye">左眼设备</label>
                </div>
                <div class="radio-item">
                  <input type="radio" id="device-right-eye" v-model="selectedDevice" value="right_eye" />
                  <label for="device-right-eye">右眼设备</label>
                </div>
              </div>
            </div>
            
            <div class="firmware-version-selector">
              <h4>固件版本</h4>
              <div class="radio-group">
                <div class="radio-item">
                  <input type="radio" id="firmware-stable" v-model="selectedFirmware" value="stable" />
                  <label for="firmware-stable">稳定版 ({{ stableVersion }})</label>
                </div>
                <div class="radio-item">
                  <input type="radio" id="firmware-beta" v-model="selectedFirmware" value="beta" />
                  <label for="firmware-beta">测试版 ({{ betaVersion }})</label>
                </div>
                <div class="radio-item">
                  <input type="radio" id="firmware-custom" v-model="selectedFirmware" value="custom" />
                  <label for="firmware-custom">自定义</label>
                </div>
              </div>
              
              <div v-if="selectedFirmware === 'custom'" class="custom-firmware">
                <button @click="selectCustomFirmware">选择固件文件</button>
                <div v-if="customFirmwarePath" class="selected-file">
                  {{ customFirmwarePath }}
                </div>
                <div v-else class="selected-file empty">
                  未选择文件
                </div>
              </div>
            </div>
          </div>
        </div>
        <template #footer>
          <button class="cancel-button" @click="cancelSelection">取消</button>
          <button 
            class="confirm-button" 
            @click="confirmSelection"
            :disabled="!isSelectionValid"
          >
            开始刷写
          </button>
        </template>
      </modal>
    </teleport>
  </template>
  
  <script setup>
  import { ref, computed, defineExpose } from 'vue';
  import Modal from './Modal.vue';
  import { invoke } from '@tauri-apps/api/core';
  
  const show = ref(false);
  const title = ref('选择固件');
  const description = ref('请选择要刷写的设备类型和固件版本');
  const stableVersion = ref('v1.0.0');
  const betaVersion = ref('v1.1.0-beta');
  const selectedDevice = ref('face');
  const selectedFirmware = ref('stable');
  const customFirmwarePath = ref('');
  const onConfirm = ref(null);
  const onCancel = ref(null);
  
  const isSelectionValid = computed(() => {
    if (selectedFirmware.value === 'custom') {
      return customFirmwarePath.value !== '';
    }
    return true;
  });
  
  async function selectCustomFirmware() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: '固件文件',
          extensions: ['bin']
        }]
      });
      
      if (selected) {
        customFirmwarePath.value = selected;
      }
    } catch (error) {
      console.error('选择文件失败:', error);
    }
  }
  
  function openSelector(config = {}) {
    title.value = config.title || '选择固件';
    description.value = config.description || '请选择要刷写的设备类型和固件版本';
    stableVersion.value = config.stableVersion || 'v1.0.0';
    betaVersion.value = config.betaVersion || 'v1.1.0-beta';
    selectedDevice.value = config.defaultDevice || 'face';
    selectedFirmware.value = config.defaultFirmware || 'stable';
    customFirmwarePath.value = '';
    onConfirm.value = config.onConfirm || null;
    onCancel.value = config.onCancel || null;
    show.value = true;
  }
  
  function confirmSelection() {
    show.value = false;
    if (typeof onConfirm.value === 'function') {
      onConfirm.value({
        deviceType: selectedDevice.value,
        firmwareType: selectedFirmware.value,
        firmwarePath: selectedFirmware.value === 'custom' ? customFirmwarePath.value : null
      });
    }
  }
  
  function cancelSelection() {
    show.value = false;
    if (typeof onCancel.value === 'function') {
      onCancel.value();
    }
  }
  
  defineExpose({
    open: openSelector,
    close: cancelSelection
  });
  </script>
  
  <style scoped>
  .firmware-selector-content {
    min-width: 450px;
  }
  
  .description {
    margin-bottom: 20px;
  }
  
  .selection-container {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }
  
  .radio-group {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-top: 10px;
  }
  
  .radio-item {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  
  .radio-item input[type="radio"] {
    margin: 0;
  }
  
  h4 {
    margin: 0;
    font-size: 16px;
    color: var(--text-color);
  }
  
  .custom-firmware {
    margin-top: 15px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  
  .selected-file {
    padding: 8px;
    background-color: var(--widget-background);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 12px;
    word-break: break-all;
  }
  
  .selected-file.empty {
    color: var(--disabled-color);
    font-style: italic;
  }
  
  .cancel-button {
    background-color: var(--widget-background);
  }
  
  .confirm-button {
    background-color: var(--highlight-color);
    color: white;
  }
  
  .confirm-button:hover:not(:disabled) {
    background-color: var(--highlight-hover);
  }
  
  .confirm-button:disabled {
    background-color: var(--disabled-color);
    cursor: not-allowed;
  }
  </style>