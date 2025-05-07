<!-- MainWindow.vue -->
<template>
    <div class="main-window">
      <div class="logo-container">
        <img class="logo" alt="PaperTrack Logo" src="../assets/logo.png" />
      </div>
      
      <div class="buttons-container">
        <button class="tracker-button" @click="$emit('open-face-tracker')">面捕界面</button>
        <button class="tracker-button" @click="$emit('open-eye-tracker')">眼追界面</button>
      </div>
      
      <div class="additional-controls">
        <button class="update-button" @click="checkForUpdates">客户端更新检查</button>
        
        <div class="instructions">
          <a href="#" class="instruction-link" @click="openFaceTrackerInstructions">
            点击查看面捕说明书
          </a>
          <a href="#" class="instruction-link" @click="openEyeTrackerInstructions">
            点击查看眼追说明书
          </a>
        </div>
        
        <button class="restart-button" @click="restartVRCFT">重启VRCFT</button>
        
        <div class="status-label">
          {{ serverStatus }}
        </div>
      </div>
    </div>
  </template>
  
  <script setup>
  import { ref } from 'vue';
  
  const serverStatus = ref('无法连接到服务器，请检查网络');
  
  function checkForUpdates() {
    // Implement update checking logic
    alert('正在检查更新...');
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
  .main-window {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: flex-start;
    width: 100%;
    max-width: 600px;
    padding: 20px;
  }
  
  .logo-container {
    width: 100%;
    height: 230px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 20px;
  }
  
  .logo {
    max-width: 100%;
    max-height: 100%;
  }
  
  .buttons-container {
    display: flex;
    justify-content: space-between;
    width: 100%;
    margin-bottom: 20px;
  }
  
  .tracker-button {
    width: 48%;
    height: 80px;
    font-size: 1.2rem;
  }
  
  .additional-controls {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 20px;
  }
  
  .update-button {
    width: 200px;
    height: 50px;
  }
  
  .instructions {
    display: flex;
    justify-content: space-between;
    width: 100%;
    padding: 0 20px;
  }
  
  .instruction-link {
    color: white;
    font-size: 1.1rem;
    font-weight: bold;
    font-style: italic;
    text-decoration: none;
  }
  
  .restart-button {
    width: 200px;
    height: 50px;
  }
  
  .status-label {
    font-style: italic;
    font-weight: bold;
    margin-top: 10px;
    text-align: center;
  }
  </style>