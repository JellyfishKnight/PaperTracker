<!-- FaceTrackerWindow.vue -->
<template>
    <div class="face-tracker-window">
      <div class="nav-bar">
        <button @click="$emit('back-to-main')">主页面</button>
        <button @click="currentPage = 'calibration'">标定页面</button>
        
        <div class="status-labels">
          <span class="status-label">{{ wifiStatus }}</span>
          <span class="status-label">{{ serialStatus }}</span>
        </div>
      </div>
  
      <!-- Main page content -->
      <div v-if="currentPage === 'main'" class="page-content">
        <div class="image-section">
          <div class="camera-view no-image">
            {{ cameraImage ? '' : '没有图像输入' }}
            <img v-if="cameraImage" :src="cameraImage" alt="Camera Feed" />
          </div>
        </div>
  
        <div class="controls-section">
          <div class="wifi-settings">
            <div class="input-group">
              <textarea 
                v-model="ssid" 
                placeholder="请输入WIFI名字（仅支持2.4ghz）"
                class="input-field"
              ></textarea>
            </div>
            <div class="input-group">
              <textarea 
                v-model="password" 
                placeholder="请输入WIFI密码"
                class="input-field"
              ></textarea>
            </div>
            <button class="send-button" @click="sendWifiSettings">发送</button>
          </div>
  
          <div class="action-buttons">
            <button @click="flashFirmware">刷写固件</button>
            <button @click="restartDevice">重启</button>
          </div>
  
          <div class="adjustments">
            <div class="slider-group">
              <label>亮度调整</label>
              <div class="slider" @click="updateSlider($event, 'brightness')">
                <div class="track" :style="{ width: brightness + '%' }"></div>
                <div class="thumb" :style="{ left: brightness + '%' }"></div>
              </div>
            </div>
  
            <div class="slider-group">
              <label>旋转角度调整</label>
              <div class="slider" @click="updateSlider($event, 'rotation')">
                <div class="track" :style="{ width: rotation / 10.8 + '%' }"></div>
                <div class="thumb" :style="{ left: rotation / 10.8 + '%' }"></div>
              </div>
            </div>
  
            <div class="option-controls">
              <div class="mode-selector">
                <label>性能模式选择</label>
                <select v-model="energyMode">
                  <option value="normal">普通模式</option>
                  <option value="eco">节能模式</option>
                  <option value="performance">性能模式</option>
                </select>
              </div>
  
              <div class="checkbox-group">
                <input type="checkbox" id="filter" v-model="useFilter">
                <label for="filter">启用滤波（减少抖动）</label>
              </div>
            </div>
          </div>
  
          <div class="ip-display">
            <label>IP地址：</label>
            <div class="ip-text">{{ ipAddress }}</div>
          </div>
        </div>
  
        <div class="log-section">
          <div class="log-header">
            <span>日志窗口：</span>
            <button class="serial-log-button" @click="showSerialLog">串口日志</button>
          </div>
          <textarea v-model="logContent" class="log-area" readonly></textarea>
        </div>
      </div>
  
      <!-- Calibration page content -->
      <div v-if="currentPage === 'calibration'" class="page-content calibration-page">
        <div class="magnification-control">
          <label>放大倍率</label>
          <span>x1</span>
          <span>x3</span>
        </div>
  
        <div class="tracking-controls">
          <div class="scroll-container">
            <div class="tracking-parameters">
              <!-- Left cheek controls -->
              <div class="parameter-row">
                <label>左脸颊</label>
                <div class="slider" @click="updateCalibrationSlider($event, 'cheekLeft')">
                  <div class="track" :style="{ width: calibration.cheekLeft + '%' }"></div>
                  <div class="thumb" :style="{ left: calibration.cheekLeft + '%' }"></div>
                </div>
                <div class="progress-bar">
                  <div class="progress-bar-fill" :style="{ width: calibration.cheekLeft + '%' }"></div>
                </div>
              </div>
  
              <!-- Right cheek controls -->
              <div class="parameter-row">
                <label>右脸颊</label>
                <div class="slider" @click="updateCalibrationSlider($event, 'cheekRight')">
                  <div class="track" :style="{ width: calibration.cheekRight + '%' }"></div>
                  <div class="thumb" :style="{ left: calibration.cheekRight + '%' }"></div>
                </div>
                <div class="progress-bar">
                  <div class="progress-bar-fill" :style="{ width: calibration.cheekRight + '%' }"></div>
                </div>
              </div>
  
              <!-- Additional parameters: jaw, mouth, tongue controls would go here -->
              <!-- I'm showing just a few as examples -->
              <div class="parameter-row">
                <label>下巴下移</label>
                <div class="slider" @click="updateCalibrationSlider($event, 'jawOpen')">
                  <div class="track" :style="{ width: calibration.jawOpen + '%' }"></div>
                  <div class="thumb" :style="{ left: calibration.jawOpen + '%' }"></div>
                </div>
                <div class="progress-bar">
                  <div class="progress-bar-fill" :style="{ width: calibration.jawOpen + '%' }"></div>
                </div>
              </div>
  
              <div class="parameter-row">
                <label>下巴左移</label>
                <div class="slider" @click="updateCalibrationSlider($event, 'jawLeft')">
                  <div class="track" :style="{ width: calibration.jawLeft + '%' }"></div>
                  <div class="thumb" :style="{ left: calibration.jawLeft + '%' }"></div>
                </div>
                <div class="progress-bar">
                  <div class="progress-bar-fill" :style="{ width: calibration.jawLeft + '%' }"></div>
                </div>
              </div>
  
              <!-- More parameters would be added here -->
            </div>
          </div>
  
          <div class="calibration-image">
            <div class="camera-view no-image">
              {{ calibrationImage ? '' : '没有图像输入' }}
              <img v-if="calibrationImage" :src="calibrationImage" alt="Calibration Feed" />
            </div>
          </div>
        </div>
      </div>
    </div>
  </template>
  
  <script setup>
  import { ref, reactive } from 'vue';
  
  // Page state
  const currentPage = ref('main');
  
  // Status indicators
  const wifiStatus = ref('面捕wifi未连接');
  const serialStatus = ref('面捕数据线未连接');
  const ipAddress = ref('');
  
  // Camera feeds
  const cameraImage = ref(null);
  const calibrationImage = ref(null);
  
  // Form inputs
  const ssid = ref('');
  const password = ref('');
  
  // Slider values
  const brightness = ref(50);
  const rotation = ref(540); // Middle value of 0-1080 range
  
  // Options
  const energyMode = ref('normal');
  const useFilter = ref(false);
  
  // Log content
  const logContent = ref('系统启动中...\n连接设备...');
  
  // Calibration values
  const calibration = reactive({
    cheekLeft: 24,
    cheekRight: 24,
    jawOpen: 24,
    jawLeft: 24,
    jawRight: 24,
    mouthLeft: 24,
    mouthRight: 24,
    tongueOut: 24,
    tongueUp: 24,
    tongueDown: 24,
    tongueLeft: 24,
    tongueRight: 24
  });
  
  // Methods
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
    
    if (sliderName === 'brightness') {
      brightness.value = percentage;
    } else if (sliderName === 'rotation') {
      rotation.value = percentage * 10.8; // Scale to 0-1080 range
    }
  }
  
  function updateCalibrationSlider(event, paramName) {
    const rect = event.currentTarget.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const percentage = Math.min(100, Math.max(0, (x / rect.width) * 100));
    
    calibration[paramName] = percentage;
  }
  
  function showSerialLog() {
    // Show serial log
    alert('显示串口日志');
  }
  </script>
  
  <style scoped>
  .face-tracker-window {
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
  
  .image-section {
    margin-bottom: 20px;
  }
  
  .camera-view {
    width: 280px;
    height: 280px;
    background-color: #333;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .controls-section {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 20px;
    margin-bottom: 20px;
  }
  
  .wifi-settings {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  
  .input-field {
    width: 100%;
    min-height: 40px;
    padding: 8px;
  }
  
  .send-button {
    width: 90px;
    height: 90px;
    align-self: flex-end;
  }
  
  .action-buttons {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  
  .adjustments {
    grid-column: span 2;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }
  
  .slider-group {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  
  .slider-group label {
    min-width: 100px;
  }
  
  .option-controls {
    display: flex;
    align-items: center;
    gap: 20px;
  }
  
  .mode-selector {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  
  .mode-selector select {
    padding: 5px;
    background-color: var(--widget-background);
    color: var(--text-color);
    border: 1px solid #3a3a3a;
  }
  
  .checkbox-group {
    display: flex;
    align-items: center;
    gap: 5px;
  }
  
  .ip-display {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  
  .ip-text {
    padding: 8px;
    background-color: var(--widget-background);
    border: 1px solid #3a3a3a;
    border-radius: 4px;
    flex-grow: 1;
    min-height: 40px;
  }
  
  .log-section {
    width: 100%;
  }
  
  .log-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 5px;
  }
  
  .log-area {
    width: 100%;
    height: 150px;
    padding: 10px;
    resize: none;
  }
  
  /* Calibration page styles */
  .calibration-page {
    display: flex;
    flex-direction: column;
  }
  
  .magnification-control {
    display: flex;
    align-items: center;
    gap: 40px;
    margin-bottom: 20px;
  }
  
  .tracking-controls {
    display: grid;
    grid-template-columns: 1fr 280px;
    gap: 20px;
  }
  
  .scroll-container {
    max-height: 500px;
    overflow-y: auto;
  }
  
  .tracking-parameters {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }
  
  .parameter-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  
  .parameter-row label {
    min-width: 90px;
  }
  
  .parameter-row .slider {
    width: 200px;
  }
  
  .parameter-row .progress-bar {
    width: 150px;
  }
  
  .calibration-image {
    display: flex;
    align-items: center;
    justify-content: center;
  }
  </style>