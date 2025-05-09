<!-- MainWindow.vue -->
<template>
    <div class="main-content">
      <!-- Logo区域 -->
      <div class="logo-container">
        <img class="logo" alt="PaperTrack Logo" src="../assets/logo.png" />
      </div>
      
      <div class="main-actions">
        <button class="action-button" @click="openEyeTrackerInstructions">
          眼追使用说明
        </button>
        <button class="action-button" @click="openFaceTrackerInstructions">
          面捕使用说明
        </button>
        <button class="action-button" @click="checkForUpdates">
          检查更新
        </button>
      </div>
  
      <div class="info-section">
        <div class="version-info">
          <span class="info-label">当前版本：</span>
          <span>1.0.0</span>
        </div>
        
        <div class="update-log">
          <span class="info-label">更新日志：</span>
          <div class="log-content">
            <p>系统启动中...</p>
            <p>连接设备...</p>
          </div>
        </div>
        
        <div class="action-container">
          <button class="restart-button" @click="restartVRCFT">重启VRCFT</button>
          
          <div class="status-label">
            {{ serverStatus }}
          </div>
        </div>
      </div>
    </div>
  </template>
  
  <script setup>
  import { ref } from 'vue';
  import { invoke } from '@tauri-apps/api/core';
  const serverStatus = ref('无法连接到服务器，请检查网络');
  
  function checkForUpdates() {
    // Implement update checking logic
    const result = invoke('check_for_updates');
  }
  
  function openFaceTrackerInstructions() {
    // Open face tracker instructions
    window.open('#', '_blank');
  }
  
  function openEyeTrackerInstructions() {
    // Open eye tracker instructions
    window.open('#', '_blank');
  }
  
  function restartVRCFT() {
    // Implement VRCFT restart logic
    alert('正在重启VRCFT...');
  }
  </script>
  
  <style scoped>
  .main-content {
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
  }
  
  .logo-container {
    width: 100%;
    height: 230px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 30px;
  }
  
  .logo {
    max-width: 100%;
    max-height: 100%;
  }
  
  .main-actions {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    gap: 20px;
    margin-bottom: 30px;
  }
  
  .action-button {
    padding: 20px;
    height: 100px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.1rem;
  }
  
  .info-section {
    background-color: var(--widget-background);
    padding: 20px;
    border-radius: 8px;
  }
  
  .version-info, .update-log {
    margin-bottom: 20px;
  }
  
  .info-label {
    font-weight: bold;
    margin-right: 10px;
  }
  
  .log-content {
    margin-top: 10px;
    background-color: rgba(0, 0, 0, 0.2);
    padding: 10px;
    border-radius: 4px;
    max-height: 150px;
    overflow-y: auto;
  }
  
  .action-container {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .restart-button {
    width: 150px;
  }
  
  .status-label {
    font-style: italic;
    font-weight: bold;
  }
  </style>