<!-- EyeTrackerWindow.vue -->
<template>
    <div class="eye-tracker-window">
      <div class="nav-bar">
        <button @click="$emit('back-to-main')">主页面</button>
        <button @click="currentPage = 'settings'">设置</button>
        
        <div class="status-labels">
          <span class="status-label">{{ serialStatus }}</span>
          <span class="status-label">{{ leftEyeWifiStatus }}</span>
          <span class="status-label">{{ rightEyeWifiStatus }}</span>
        </div>
      </div>
  
      <!-- Main tracking page content -->
      <div v-if="currentPage === 'tracking'" class="page-content">
        <div class="eye-tracking-container">
          <!-- Left eye section -->
          <div class="eye-section">
            <h2>左眼跟踪</h2>
            
            <div class="calibration-buttons">
              <button @click="calibrateLeftEye">左眼校准</button>
              <button @click="centerLeftEye">左眼中心</button>
            </div>
            
            <div class="eye-position-frame">
              <!-- This would contain a canvas or visualization -->
            </div>
            
            <div class="eye-openness">
              <label>左眼开合度</label>
              <div class="progress-bar">
                <div 
                  class="progress-bar-fill" 
                  :style="{ width: leftEyeOpenness + '%' }"
                ></div>
              </div>
            </div>
          </div>
  
          <!-- Right eye section -->
          <div class="eye-section">
            <h2>右眼跟踪</h2>
            
            <div class="calibration-buttons">
              <button @click="calibrateRightEye">右眼校准</button>
              <button @click="centerRightEye">右眼中心</button>
            </div>
            
            <div class="eye-position-frame">
              <!-- This would contain a canvas or visualization -->
            </div>
            
            <div class="eye-openness">
              <label>右眼开合度</label>
              <div class="progress-bar">
                <div 
                  class="progress-bar-fill" 
                  :style="{ width: rightEyeOpenness + '%' }"
                ></div>
              </div>
            </div>
          </div>
        </div>
      </div>
  
      <!-- Settings page content -->
      <div v-if="currentPage === 'settings'" class="page-content settings-page">
        <div class="camera-views">
          <div class="eye-image left-eye">
            <div class="no-image">
              {{ leftEyeImage ? '' : '没有图像输入' }}
              <img v-if="leftEyeImage" :src="leftEyeImage" alt="Left Eye Camera Feed" />
            </div>
          </div>
          
          <div class="eye-image right-eye">
            <div class="no-image">
              {{ rightEyeImage ? '' : '没有图像输入' }}
              <img v-if="rightEyeImage" :src="rightEyeImage" alt="Right Eye Camera Feed" />
            </div>
          </div>
        </div>
  
        <div class="settings-controls">
          <div class="wifi-settings">
            <div class="input-group">
              <label>SSID</label>
              <textarea v-model="ssid" class="input-field"></textarea>
            </div>
            
            <div class="input-group">
              <label>密码</label>
              <textarea v-model="password" class="input-field"></textarea>
            </div>
            
            <button class="send-button" @click="sendWifiSettings">发送</button>
          </div>
          
          <div class="action-buttons">
            <button @click="restartDevice">重启</button>
            <button @click="flashFirmware">刷写固件</button>
          </div>
          
          <div class="eye-ip-settings">
            <div class="input-group">
              <label>左眼IP</label>
              <textarea v-model="leftEyeIP" class="input-field"></textarea>
            </div>
            
            <div class="input-group">
              <label>右眼IP</label>
              <textarea v-model="rightEyeIP" class="input-field"></textarea>
            </div>
            
            <div class="mode-selector">
              <label>模式选择</label>
              <select v-model="energyMode">
                <option value="normal">普通模式</option>
                <option value="eco">节能模式</option>
                <option value="performance">性能模式</option>
              </select>
            </div>
          </div>
          
          <div class="brightness-controls">
            <div class="slider-group">
              <label>左眼补光</label>
              <div class="slider" @click="updateSlider($event, 'leftBrightness')">
                <div class="track" :style="{ width: leftBrightness + '%' }"></div>
                <div class="thumb" :style="{ left: leftBrightness + '%' }"></div>
              </div>
            </div>
            
            <div class="slider-group">
              <label>右眼补光</label>
              <div class="slider" @click="updateSlider($event, 'rightBrightness')">
                <div class="track" :style="{ width: rightBrightness + '%' }"></div>
                <div class="thumb" :style="{ left: rightBrightness + '%' }"></div>
              </div>
            </div>
          </div>
          
          <div class="rotation-controls">
            <div class="slider-group">
              <label>左眼旋转角度</label>
              <div class="slider" @click="updateSlider($event, 'leftRotation')">
                <div class="track" :style="{ width: leftRotation / 10.8 + '%' }"></div>
                <div class="thumb" :style="{ left: leftRotation / 10.8 + '%' }"></div>
              </div>
            </div>
            
            <div class="slider-group">
              <label>右眼旋转角度</label>
              <div class="slider" @click="updateSlider($event, 'rightRotation')">
                <div class="track" :style="{ width: rightRotation / 10.8 + '%' }"></div>
                <div class="thumb" :style="{ left: rightRotation / 10.8 + '%' }"></div>
              </div>
            </div>
          </div>
        </div>
        
        <div class="log-area">
          <textarea v-model="logContent" readonly></textarea>
        </div>
      </div>
    </div>
  </template>
  
  <script setup>
  import { ref } from 'vue';
  
  // Page state
  const currentPage = ref('tracking');
  
  // Status indicators
  const serialStatus = ref('当前无串口连接');
  const leftEyeWifiStatus = ref('左眼WIFI未连接');
  const rightEyeWifiStatus = ref('右眼WIFI未连接');
  
  // Camera feeds
  const leftEyeImage = ref(null);
  const rightEyeImage = ref(null);
  
  // Form inputs
  const ssid = ref('');
  const password = ref('');
  const leftEyeIP = ref('');
  const rightEyeIP = ref('');
  
  // Progress indicators
  const leftEyeOpenness = ref(0);
  const rightEyeOpenness = ref(0);
  
  // Slider values
  const leftBrightness = ref(50);
  const rightBrightness = ref(50);
  const leftRotation = ref(540); // Middle value of 0-1080 range
  const rightRotation = ref(540); // Middle value of 0-1080 range
  
  // Options
  const energyMode = ref('normal');
  
  // Log content
  const logContent = ref('系统启动中...\n连接设备...');
  
  // Methods
  function calibrateLeftEye() {
    logContent.value += '\n开始左眼校准...';
    // Implement left eye calibration logic
  }
  
  function centerLeftEye() {
    logContent.value += '\n设置左眼中心...';
    // Implement left eye centering logic
  }
  
  function calibrateRightEye() {
    logContent.value += '\n开始右眼校准...';
    // Implement right eye calibration logic
  }
  
  function centerRightEye() {
    logContent.value += '\n设置右眼中心...';
    // Implement right eye centering logic
  }
  
  function sendWifiSettings() {
    logContent.value += `\n正在发送WiFi设置：${ssid.value}...`;
    // Implement actual sending logic
  }
  
  function flashFirmware() {
    logContent.value += '\n开始刷写固件...';
    // Implement firmware flashing logic
  }
  
  function restartDevice() {
    logContent.value += '\n重启设备...';
    // Implement restart logic
  }
  
  function updateSlider(event, sliderName) {
    const rect = event.currentTarget.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const percentage = Math.min(100, Math.max(0, (x / rect.width) * 100));
    
    switch(sliderName) {
      case 'leftBrightness':
        leftBrightness.value = percentage;
        break;
      case 'rightBrightness':
        rightBrightness.value = percentage;
        break;
      case 'leftRotation':
        leftRotation.value = percentage * 10.8; // Scale to 0-1080 range
        break;
      case 'rightRotation':
        rightRotation.value = percentage * 10.8; // Scale to 0-1080 range
        break;
    }
  }
  </script>
  
  <style scoped>
  .eye-tracker-window {
    width: 100%;
    max-width: 900px;
    display: flex;
    flex-direction: column;
  }
  
  .nav-bar {
    display: flex;
    align-items: center;
    margin-bottom: 20px;
    padding: 10px 0;
  }
  
  .nav-bar button {
    margin-right: 15px;
  }
  
  .status-labels {
    margin-left: auto;
    display: flex;
    gap: 20px;
  }
  
  .page-content {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }
  
  /* Eye tracking page styles */
  .eye-tracking-container {
    display: flex;
    flex-wrap: wrap;
    gap: 30px;
    justify-content: space-between;
  }
  
  .eye-section {
    flex: 1;
    min-width: 300px;
    display: flex;
    flex-direction: column;
    gap: 15px;
  }
  
  .eye-section h2 {
    font-size: 1.2rem;
    font-weight: bold;
    margin-bottom: 10px;
  }
  
  .calibration-buttons {
    display: flex;
    gap: 10px;
  }
  
  .eye-position-frame {
    width: 250px;
    height: 250px;
    border: 1px solid var(--text-color);
    background-color: var(--widget-background);
    margin: 0 auto;
  }
  
  .eye-openness {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  
  /* Settings page styles */
  .settings-page {
    display: grid;
    grid-template-columns: 1fr;
    gap: 20px;
  }
  
  .camera-views {
    display: flex;
    gap: 20px;
    justify-content: flex-start;
    flex-wrap: wrap;
  }
  
  .eye-image {
    width: 261px;
    height: 261px;
  }
  
  .no-image {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: #333;
    color: #aaa;
  }
  
  .settings-controls {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 20px;
  }
  
  .wifi-settings {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  
  .input-group {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  
  .input-group label {
    min-width: 70px;
  }
  
  .input-field {
    width: 100%;
    min-height: 40px;
    padding: 8px;
  }
  
  .send-button {
    width: 100px;
    height: 90px;
    align-self: flex-end;
  }
  
  .action-buttons {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  
  .eye-ip-settings {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  
  .mode-selector {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 10px;
  }
  
  .mode-selector select {
    padding: 5px;
    background-color: var(--widget-background);
    color: var(--text-color);
    border: 1px solid #3a3a3a;
  }
  
  .brightness-controls, .rotation-controls {
    display: flex;
    flex-direction: column;
    gap: 15px;
  }
  
  .slider-group {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  
  .slider-group label {
    min-width: 100px;
  }
  
  .log-area {
    width: 100%;
    margin-top: 20px;
  }
  
  .log-area textarea {
    width: 100%;
    height: 150px;
    padding: 10px;
    resize: none;
  }
  </style>