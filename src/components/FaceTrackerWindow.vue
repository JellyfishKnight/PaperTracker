<!-- FaceTrackerWindow.vue -->
<template>
  <div class="face-tracker-window">
    <!-- 子导航栏 -->
    <div class="sub-nav-bar">
      <button 
        :class="{ active: currentPage === 'main' }"
        @click="currentPage = 'main'"
      >
        主界面
      </button>
      <button
        :class="{ active: currentPage === 'calibration' }"
        @click="currentPage = 'calibration'"
      >
        标定界面
      </button>
      
      <div class="status-labels">
        <span class="status-label">{{ wifiStatus }}</span>
        <span class="status-label">{{ serialStatus }}</span>
      </div>
    </div>

    <!-- 主页面内容 -->
    <div v-if="currentPage === 'main'" class="page-content">
      <div class="main-layout">
        <div class="left-section">
          <div class="image-section">
            <div class="camera-view no-image">
              {{ cameraImage ? '' : '没有图像输入' }}
              <img v-if="cameraImage" :src="cameraImage" alt="Camera Feed" />
            </div>
          </div>
          
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
          </div>
        </div>
        
        <div class="right-section">
          <div class="action-buttons-container">
            <button class="action-button" @click="flashFirmware">刷写固件</button>
            <button class="action-button" @click="restartDevice">重启</button>
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
  
          <div class="ip-section">
            <div class="ip-display">
              <label>IP地址：</label>
              <div class="ip-text">{{ ipAddress }}</div>
            </div>
            
            <!-- 发送按钮移动到这里 -->
            <button class="send-button" @click="sendWifiSettings">发送</button>
          </div>
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

    <!-- 标定页面内容 -->
    <div v-if="currentPage === 'calibration'" class="page-content calibration-page">
      <!-- Calibration page content -->
      <div class="magnification-control">
        <label>放大倍率</label>
        <span>x1</span>
        <span>x3</span>
      </div>

      <div class="tracking-controls">
        <div class="scroll-container">
          <div class="tracking-parameters">
            <!-- 参数调整行 -->
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

            <!-- 其他参数行 (略去重复内容) -->
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

            <div class="parameter-row">
              <label>下巴右移</label>
              <div class="slider" @click="updateCalibrationSlider($event, 'jawRight')">
                <div class="track" :style="{ width: calibration.jawRight + '%' }"></div>
                <div class="thumb" :style="{ left: calibration.jawRight + '%' }"></div>
              </div>
              <div class="progress-bar">
                <div class="progress-bar-fill" :style="{ width: calibration.jawRight + '%' }"></div>
              </div>
            </div>

            <div class="parameter-row">
              <label>嘴左移</label>
              <div class="slider" @click="updateCalibrationSlider($event, 'mouthLeft')">
                <div class="track" :style="{ width: calibration.mouthLeft + '%' }"></div>
                <div class="thumb" :style="{ left: calibration.mouthLeft + '%' }"></div>
              </div>
              <div class="progress-bar">
                <div class="progress-bar-fill" :style="{ width: calibration.mouthLeft + '%' }"></div>
              </div>
            </div>

            <div class="parameter-row">
              <label>嘴右移</label>
              <div class="slider" @click="updateCalibrationSlider($event, 'mouthRight')">
                <div class="track" :style="{ width: calibration.mouthRight + '%' }"></div>
                <div class="thumb" :style="{ left: calibration.mouthRight + '%' }"></div>
              </div>
              <div class="progress-bar">
                <div class="progress-bar-fill" :style="{ width: calibration.mouthRight + '%' }"></div>
              </div>
            </div>
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

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import deviceService from '../functional/deviceService';
import messageService from '../functional/pop_window/messageService';
import { invoke, Channel } from '@tauri-apps/api/core';

type PageType = 'main' | 'calibration';
type EnergyMode = 'normal' | 'eco' | 'performance';

interface CalibrationValues {
  cheekLeft: number;
  cheekRight: number;
  jawOpen: number;
  jawLeft: number;
  jawRight: number;
  mouthLeft: number;
  mouthRight: number;
  tongueOut: number;
  tongueUp: number;
  tongueDown: number;
  tongueLeft: number;
  tongueRight: number;
}

// 页面状态
const currentPage = ref<PageType>('main');

// 状态指示器
const wifiStatus = ref<string>('面捕wifi未连接');
const serialStatus = ref<string>('面捕数据线未连接');
const ipAddress = ref<string>('');

// 相机画面
const cameraImage = ref<string | null>(null);
const calibrationImage = ref<string | null>(null);

// 表单输入
const ssid = ref<string>('');
const password = ref<string>('');

// 滑块值
const brightness = ref<number>(50);
const rotation = ref<number>(540); // 0-1080范围的中间值

// 选项
const energyMode = ref<EnergyMode>('normal');
const useFilter = ref<boolean>(false);

// 日志内容
const logContent = ref<string>('系统启动中...\n连接设备...');

// 校准值
const calibration = reactive<CalibrationValues>({
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

// 方法
function sendWifiSettings(): void {
  // 读取SSID和密码
  invoke('write_wifi_info', { ssid: ssid.value, password: password.value })
    .then(() => {
      messageService.info("设置WIFI成功，请重启设备");
    })
    .catch((error) => {
      messageService.error("设置WIFI失败: " + error);
    });
}

function flashFirmware(): void {
  deviceService.flashESP32();
}

function restartDevice(): void {
  deviceService.restartESP32();
}

function updateSlider(event: MouseEvent, sliderName: 'brightness' | 'rotation'): void {
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
  const x = event.clientX - rect.left;
  const percentage = Math.min(100, Math.max(0, (x / rect.width) * 100));
  
  if (sliderName === 'brightness') {
    brightness.value = percentage;
  } else if (sliderName === 'rotation') {
    rotation.value = percentage * 10.8; // 缩放到0-1080范围
  }
}

type CalibrationParam = keyof CalibrationValues;

function updateCalibrationSlider(event: MouseEvent, paramName: CalibrationParam): void {
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
  const x = event.clientX - rect.left;
  const percentage = Math.min(100, Math.max(0, (x / rect.width) * 100));
  
  calibration[paramName] = percentage;
}

function showSerialLog(): void {
  // 显示串口日志
  alert('显示串口日志');
}

onMounted(() => {
  const channel = new Channel<Uint8Array>();
  channel.onmessage = (message) => {
    
  };
  invoke('start_face_stream', { channel }).then(() => {})
    .catch((error) => {
      messageService.error("启动相机流失败: " + error);
    });
});

</script>

<style scoped>
.face-tracker-window {
  width: 100%;
  max-width: 900px;
  display: flex;
  flex-direction: column;
}

.sub-nav-bar {
  display: flex;
  align-items: center;
  margin-bottom: 20px;
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
  gap: 20px;
}

/* Additional CSS... */
/* 主布局样式 */
.main-layout {
  display: flex;
  gap: 20px;
}

.left-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.right-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.image-section {
  position: relative;
}

.camera-view {
  width: 280px;
  height: 280px;
  background-color: #333;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

/* 按钮容器样式 */
.action-buttons-container {
  display: flex;
  flex-direction: column;
  gap: 20px;
  margin-bottom: 20px;
}

.action-button {
  width: 100%;
  padding: 15px;
  font-size: 1.1rem;
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

/* 更新发送按钮样式和位置 */
.ip-section {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.send-button {
  width: 100%;
  height: 50px;
  padding: 10px;
  font-size: 1.1rem;
  background-color: var(--widget-background);
  color: var(--text-color);
  border: 1px solid var(--border-color);
  cursor: pointer;
  transition: all 0.3s ease;
}

.send-button:hover {
  border-bottom-color: var(--highlight-hover);
  color: #FFFFFF;
}

.adjustments {
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

/* 标定页面样式 */
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