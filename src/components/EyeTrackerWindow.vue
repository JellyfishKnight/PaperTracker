<!-- EyeTrackerWindow.vue - 使用可复用滑动条组件 -->
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

    <!-- 追踪页面内容 -->
    <div v-if="currentPage === 'tracking'" class="page-content">
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
    <div v-if="currentPage === 'settings'" class="page-content settings-page">
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
          
          <!-- 使用可复用滑动条组件替换原来的滑动条 -->
          <div class="adjustment-controls">
            <DraggableSlider
              v-model="leftBrightness"
              label="左眼补光"
              unit="%"
              :min="0"
              :max="100"
              :step="1"
              :throttle-ms="50"
              @input="handleLeftBrightnessRealTimeUpdate"
              @change="handleLeftBrightnessChange"
            />
            
            <DraggableSlider
              v-model="rightBrightness"
              label="右眼补光"
              unit="%"
              :min="0"
              :max="100"
              :step="1"
              :throttle-ms="50"
              @input="handleRightBrightnessRealTimeUpdate"
              @change="handleRightBrightnessChange"
            />
            
            <DraggableSlider
              v-model="leftRotation"
              label="左眼旋转角度"
              unit="°"
              :min="0"
              :max="360"
              :step="1"
              :throttle-ms="50"
              @input="handleLeftRotationRealTimeUpdate"
              @change="handleLeftRotationChange"
            />
            
            <DraggableSlider
              v-model="rightRotation"
              label="右眼旋转角度"
              unit="°"
              :min="0"
              :max="360"
              :step="1"
              :throttle-ms="50"
              @input="handleRightRotationRealTimeUpdate"
              @change="handleRightRotationChange"
            />
          </div>
        </div>
      </div>
      
      <div class="log-area">
        <textarea v-model="logContent" readonly></textarea>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import DraggableSlider from './DraggableSlider.vue'; // 导入可复用滑动条组件
import deviceService from '../functional/deviceService';
import { invoke } from '@tauri-apps/api/core';
import messageService from '../functional/pop_window/messageService';

type PageType = 'tracking' | 'settings';
type EnergyMode = 'normal' | 'eco' | 'performance';

// 页面状态
const currentPage = ref<PageType>('tracking');

// 状态指示器
const serialStatus = ref<string>('当前无串口连接');
const leftEyeWifiStatus = ref<string>('左眼WIFI未连接');
const rightEyeWifiStatus = ref<string>('右眼WIFI未连接');

// 相机画面
const leftEyeImage = ref<string | null>(null);
const rightEyeImage = ref<string | null>(null);

// 表单输入
const ssid = ref<string>('');
const password = ref<string>('');
const leftEyeIP = ref<string>('');
const rightEyeIP = ref<string>('');

// 进度指示器
const leftEyeOpenness = ref<number>(30);
const rightEyeOpenness = ref<number>(70);

// 滑块值 - 使用更合理的初始值和范围
const leftBrightness = ref<number>(50);     // 0-100%
const rightBrightness = ref<number>(50);    // 0-100%
const leftRotation = ref<number>(0);        // 0-360°
const rightRotation = ref<number>(0);       // 0-360°

// 选项
const energyMode = ref<EnergyMode>('normal');

// 日志内容
const logContent = ref<string>('系统启动中...\n连接设备...');

// 添加日志的辅助函数
function appendLog(message: string): void {
  const timestamp = new Date().toLocaleTimeString();
  logContent.value += `\n[${timestamp}] ${message}`;
  
  // 自动滚动到底部
  setTimeout(() => {
    const logArea = document.querySelector('.log-area textarea') as HTMLTextAreaElement;
    if (logArea) {
      logArea.scrollTop = logArea.scrollHeight;
    }
  }, 10);
}

// 左眼亮度处理函数
function handleLeftBrightnessRealTimeUpdate(value: number): void {
  // 实时更新左眼亮度
  invoke('set_left_brightness', { brightness: Math.round(value) })
    .catch((error) => {
      console.error(`左眼亮度实时调整失败: ${error}`);
    });
}

function handleLeftBrightnessChange(value: number): void {
  appendLog(`左眼补光调整为: ${Math.round(value)}%`);
}

// 右眼亮度处理函数
function handleRightBrightnessRealTimeUpdate(value: number): void {
  // 实时更新右眼亮度
  invoke('set_right_brightness', { brightness: Math.round(value) })
    .catch((error) => {
      console.error(`右眼亮度实时调整失败: ${error}`);
    });
}

function handleRightBrightnessChange(value: number): void {
  appendLog(`右眼补光调整为: ${Math.round(value)}%`);
}

// 左眼旋转角度处理函数
function handleLeftRotationRealTimeUpdate(value: number): void {
  // 实时更新左眼旋转角度，提供连续旋转效果
  invoke('set_rotation', { rotation: value, deviceType: 2 }) // 2 = 左眼
    .catch((error) => {
      console.error(`左眼旋转角度实时调整失败: ${error}`);
    });
}

function handleLeftRotationChange(value: number): void {
  appendLog(`左眼旋转角度调整为: ${Math.round(value)}°`);
}

// 右眼旋转角度处理函数
function handleRightRotationRealTimeUpdate(value: number): void {
  // 实时更新右眼旋转角度，提供连续旋转效果
  invoke('set_rotation', { rotation: value, deviceType: 3 }) // 3 = 右眼
    .catch((error) => {
      console.error(`右眼旋转角度实时调整失败: ${error}`);
    });
}

function handleRightRotationChange(value: number): void {
  appendLog(`右眼旋转角度调整为: ${Math.round(value)}°`);
}

// 眼部校准相关方法
function calibrateLeftEye(): void {
  appendLog('开始左眼校准...');
  // 实现左眼校准逻辑
}

function centerLeftEye(): void {
  appendLog('设置左眼中心...');
  // 实现左眼中心逻辑
}

function calibrateRightEye(): void {
  appendLog('开始右眼校准...');
  // 实现右眼校准逻辑
}

function centerRightEye(): void {
  appendLog('设置右眼中心...');
  // 实现右眼中心逻辑
}

// 其他功能函数
function sendWifiSettings(): void {
  // 读取SSID和密码
  invoke('write_wifi_info', { ssid: ssid.value, password: password.value })
    .then(() => {
      messageService.info("设置WIFI成功，请重启设备");
      appendLog("设置WIFI成功，请重启设备");
    })
    .catch((error) => {
      messageService.error("设置WIFI失败: " + error);
      appendLog(`设置WIFI失败: ${error}`);
    });
}

function flashFirmware(): void {
  appendLog("开始刷写固件...");
  deviceService.flashESP32();
}

function restartDevice(): void {
  appendLog("正在重启设备...");
  deviceService.restartESP32();
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

.no-image {
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #333;
  color: #aaa;
  text-align: center;
  width: 100%;
  height: 100%;
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
  gap: 15px;
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

.status-label {
  font-style: italic;
  font-weight: bold;
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
  background-color: var(--widget-background);
  color: var(--text-color);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-family: monospace;
  font-size: 0.9rem;
}
</style>