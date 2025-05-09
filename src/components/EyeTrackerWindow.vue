<!-- EyeTrackerWindow.vue -->
<template>
  <div class="eye-tracker-window">
    <!-- 子导航栏 -->
    <div class="sub-nav-bar">
      <button 
        :class="{ active: currentPage === 'tracking' }"
        @click="currentPage = 'tracking'"
      >
        追踪界面
      </button>
      <button 
        :class="{ active: currentPage === 'settings' }"
        @click="currentPage = 'settings'"
      >
        设置界面
      </button>
      
      <div class="status-labels">
        <span class="status-label">{{ serialStatus }}</span>
        <span class="status-label">{{ leftEyeWifiStatus }}</span>
        <span class="status-label">{{ rightEyeWifiStatus }}</span>
      </div>
    </div>

    <!-- 主追踪页面内容 -->
    <div v-if="currentPage === 'settings'" class="page-content">
      <div class="eye-tracking-layout">
        <!-- 左眼部分 -->
        <div class="eye-section">
          <h2>左眼跟踪</h2>
          
          <div class="eye-position-frame">
            <!-- 这里将包含画布或可视化 -->
          </div>
          
          <div class="control-group">
            <div class="calibration-buttons">
              <button @click="calibrateLeftEye">左眼校准</button>
              <button @click="centerLeftEye">左眼中心</button>
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
        </div>

        <!-- 右眼部分 -->
        <div class="eye-section">
          <h2>右眼跟踪</h2>
          
          <div class="eye-position-frame">
            <!-- 这里将包含画布或可视化 -->
          </div>
          
          <div class="control-group">
            <div class="calibration-buttons">
              <button @click="calibrateRightEye">右眼校准</button>
              <button @click="centerRightEye">右眼中心</button>
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
    </div>

    <!-- 设置页面内容 -->
    <div v-if="currentPage === 'tracking'" class="page-content settings-page">
      <div class="settings-layout">
        <div class="left-column">
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
          
          <div class="wifi-settings">
            <div class="input-group">
              <label>SSID</label>
              <textarea v-model="ssid" class="input-field"></textarea>
            </div>
            
            <div class="input-group">
              <label>密码</label>
              <textarea v-model="password" class="input-field"></textarea>
            </div>
          </div>
        </div>
        
        <div class="right-column">
          <div class="action-panel">
            <button class="send-button" @click="sendWifiSettings">发送</button>
            
            <div class="action-buttons">
              <button @click="restartDevice">重启</button>
              <button @click="flashFirmware">刷写固件</button>
            </div>
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
          
          <div class="adjustment-controls">
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
      </div>
      
      <div class="log-area">
        <textarea v-model="logContent" readonly></textarea>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';

// 页面状态
const currentPage = ref('tracking');

// 状态指示器
const serialStatus = ref('当前无串口连接');
const leftEyeWifiStatus = ref('左眼WIFI未连接');
const rightEyeWifiStatus = ref('右眼WIFI未连接');

// 相机画面
const leftEyeImage = ref(null);
const rightEyeImage = ref(null);

// 表单输入
const ssid = ref('');
const password = ref('');
const leftEyeIP = ref('');
const rightEyeIP = ref('');

// 进度指示器
const leftEyeOpenness = ref(30);
const rightEyeOpenness = ref(70);

// 滑块值
const leftBrightness = ref(50);
const rightBrightness = ref(50);
const leftRotation = ref(540); // 0-1080范围的中间值
const rightRotation = ref(540); // 0-1080范围的中间值

// 选项
const energyMode = ref('normal');

// 日志内容
const logContent = ref('系统启动中...\n连接设备...');

// 方法
function calibrateLeftEye() {
  logContent.value += '\n开始左眼校准...';
  // 实现左眼校准逻辑
}

function centerLeftEye() {
  logContent.value += '\n设置左眼中心...';
  // 实现左眼中心逻辑
}

function calibrateRightEye() {
  logContent.value += '\n开始右眼校准...';
  // 实现右眼校准逻辑
}

function centerRightEye() {
  logContent.value += '\n设置右眼中心...';
  // 实现右眼中心逻辑
}

function sendWifiSettings() {
  logContent.value += `\n正在发送WiFi设置：${ssid.value}...`;
  // 实现实际发送逻辑
}

function flashFirmware() {
  logContent.value += '\n开始刷写固件...';
  // 实现固件刷写逻辑
}

function restartDevice() {
  logContent.value += '\n重启设备...';
  // 实现重启逻辑
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
      leftRotation.value = percentage * 10.8; // 缩放到0-1080范围
      break;
    case 'rightRotation':
      rightRotation.value = percentage * 10.8; // 缩放到0-1080范围
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

.sub-nav-bar {
  display: flex;
  align-items: center;
  margin-bottom: 15px;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--border-color);
}

.sub-nav-bar button {
  margin-right: 15px;
  padding: 8px 12px;
  border-bottom: 2px solid transparent;
}

.sub-nav-bar button.active {
  border-bottom-color: var(--highlight-color);
  color: var(--highlight-color);
}

.status-labels {
  margin-left: auto;
  display: flex;
  gap: 20px;
}

.page-content {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

/* 眼部追踪页面样式 */
.eye-tracking-layout {
  display: flex;
  gap: 20px;
}

.eye-section {
  flex: 1;
  min-width: 250px;
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.eye-section h2 {
  font-size: 1.2rem;
  font-weight: bold;
  margin-bottom: 5px;
}

.control-group {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.calibration-buttons {
  display: flex;
  gap: 10px;
}

.calibration-buttons button {
  flex: 1;
  padding: 8px;
}

.eye-position-frame {
  width: 220px;
  height: 220px;
  border: 1px solid var(--text-color);
  background-color: var(--widget-background);
  margin: 0 auto;
}

.eye-openness {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

/* 设置页面样式 */
.settings-layout {
  display: flex;
  gap: 20px;
}

.left-column, .right-column {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.camera-views {
  display: flex;
  gap: 15px;
  flex-wrap: wrap;
}

.eye-image {
  width: 200px;
  height: 200px;
  border: 1px solid var(--border-color);
}

.action-panel {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.send-button {
  width: 100%;
  height: 50px;
  font-size: 1.1rem;
  background-color: var(--widget-background);
  border-bottom: 2px solid transparent;
}

.send-button:hover {
  border-bottom-color: var(--highlight-hover);
  color: #FFFFFF;
}

.action-buttons {
  display: flex;
  gap: 15px;
}

.action-buttons button {
  flex: 1;
  height: 40px;
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
  min-height: 35px;
  padding: 8px;
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
  margin-top: 5px;
}

.adjustment-controls {
  display: flex;
  flex-direction: column;
  gap: 12px;
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
  margin-top: 15px;
}

.log-area textarea {
  width: 100%;
  height: 120px;
  padding: 10px;
  resize: none;
}
</style>