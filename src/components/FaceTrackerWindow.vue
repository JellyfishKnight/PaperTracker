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
            <div class="camera-view" :class="{ 'no-image': !cameraImage }">
              <span v-if="!cameraImage">没有图像输入</span>
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

            <!-- 其他参数行 -->
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

            <div class="parameter-row">
              <label>舌头伸出</label>
              <div class="slider" @click="updateCalibrationSlider($event, 'tongueOut')">
                <div class="track" :style="{ width: calibration.tongueOut + '%' }"></div>
                <div class="thumb" :style="{ left: calibration.tongueOut + '%' }"></div>
              </div>
              <div class="progress-bar">
                <div class="progress-bar-fill" :style="{ width: calibration.tongueOut + '%' }"></div>
              </div>
            </div>

            <div class="parameter-row">
              <label>舌头上移</label>
              <div class="slider" @click="updateCalibrationSlider($event, 'tongueUp')">
                <div class="track" :style="{ width: calibration.tongueUp + '%' }"></div>
                <div class="thumb" :style="{ left: calibration.tongueUp + '%' }"></div>
              </div>
              <div class="progress-bar">
                <div class="progress-bar-fill" :style="{ width: calibration.tongueUp + '%' }"></div>
              </div>
            </div>

            <div class="parameter-row">
              <label>舌头下移</label>
              <div class="slider" @click="updateCalibrationSlider($event, 'tongueDown')">
                <div class="track" :style="{ width: calibration.tongueDown + '%' }"></div>
                <div class="thumb" :style="{ left: calibration.tongueDown + '%' }"></div>
              </div>
              <div class="progress-bar">
                <div class="progress-bar-fill" :style="{ width: calibration.tongueDown + '%' }"></div>
              </div>
            </div>

            <div class="parameter-row">
              <label>舌头左移</label>
              <div class="slider" @click="updateCalibrationSlider($event, 'tongueLeft')">
                <div class="track" :style="{ width: calibration.tongueLeft + '%' }"></div>
                <div class="thumb" :style="{ left: calibration.tongueLeft + '%' }"></div>
              </div>
              <div class="progress-bar">
                <div class="progress-bar-fill" :style="{ width: calibration.tongueLeft + '%' }"></div>
              </div>
            </div>

            <div class="parameter-row">
              <label>舌头右移</label>
              <div class="slider" @click="updateCalibrationSlider($event, 'tongueRight')">
                <div class="track" :style="{ width: calibration.tongueRight + '%' }"></div>
                <div class="thumb" :style="{ left: calibration.tongueRight + '%' }"></div>
              </div>
              <div class="progress-bar">
                <div class="progress-bar-fill" :style="{ width: calibration.tongueRight + '%' }"></div>
              </div>
            </div>
          </div>
        </div>

        <div class="calibration-image">
          <div class="camera-view" :class="{ 'no-image': !calibrationImage }">
            <span v-if="!calibrationImage">没有图像输入</span>
            <img v-if="calibrationImage" :src="calibrationImage" alt="Calibration Feed" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue';
import deviceService from '../functional/deviceService';
import messageService from '../functional/pop_window/messageService';
import { invoke, Channel } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

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

// 定义消息类型
interface ImageMessage {
  type: 'image';
  data: string;  // base64 图像数据
  device?: string;  // 可选：设备标识
}

interface LogMessage {
  type: 'log';
  data: string;
}

interface StatusMessage {
  type: 'status';
  data: {
    wifi?: string;
    serial?: string;
    ip?: string;
    battery?: number;
    brightness?: number;
  };
}

type Message = ImageMessage | LogMessage | StatusMessage;

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
const logContent = ref<string>('系统启动中...\n连接设备...\n');

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

// 保存 channel 引用，用于清理
let imageChannel: Channel<Uint8Array> | null = null;

// 添加日志的辅助函数
function appendLog(message: string): void {
  const timestamp = new Date().toLocaleTimeString();
  logContent.value += `[${timestamp}] ${message}\n`;
  
  // 自动滚动到底部
  const logArea = document.querySelector('.log-area') as HTMLTextAreaElement;
  if (logArea) {
    logArea.scrollTop = logArea.scrollHeight;
  }
}

// 方法
function sendWifiSettings(): void {
  if (!ssid.value || !password.value) {
    messageService.warning("请输入WIFI名称和密码");
    return;
  }
  
  // 读取SSID和密码
  invoke('write_wifi_info', { ssid: ssid.value, password: password.value })
    .then(() => {
      messageService.info("设置WIFI成功，请重启设备");
      appendLog(`设置WIFI成功 - SSID: ${ssid.value}`);
    })
    .catch((error) => {
      messageService.error("设置WIFI失败: " + error);
      appendLog(`设置WIFI失败: ${error}`);
    });
}

function flashFirmware(): void {
  appendLog("开始刷写固件...");
  deviceService.flashESP32()
    .then(() => {
      appendLog("固件刷写成功");
    })
    .catch((error) => {
      appendLog(`固件刷写失败: ${error}`);
    });
}

function restartDevice(): void {
  appendLog("正在重启设备...");
  deviceService.restartESP32()
    .then(() => {
      appendLog("设备重启命令已发送");
    })
    .catch((error) => {
      appendLog(`设备重启失败: ${error}`);
    });
}

function updateSlider(event: MouseEvent, sliderName: 'brightness' | 'rotation'): void {
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
  const x = event.clientX - rect.left;
  const percentage = Math.min(100, Math.max(0, (x / rect.width) * 100));
  
  if (sliderName === 'brightness') {
    brightness.value = percentage;
    // 发送亮度调整命令
    invoke('set_brightness', { value: Math.round(percentage) })
      .catch((error) => {
        appendLog(`亮度调整失败: ${error}`);
      });
  } else if (sliderName === 'rotation') {
    rotation.value = percentage * 10.8; // 缩放到0-1080范围
    // 发送旋转角度调整命令
    invoke('set_rotation', { value: Math.round(rotation.value) })
      .catch((error) => {
        appendLog(`旋转角度调整失败: ${error}`);
      });
  }
}

type CalibrationParam = keyof CalibrationValues;

function updateCalibrationSlider(event: MouseEvent, paramName: CalibrationParam): void {
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
  const x = event.clientX - rect.left;
  const percentage = Math.min(100, Math.max(0, (x / rect.width) * 100));
  
  calibration[paramName] = percentage;
  
  // 发送校准参数更新
  invoke('update_calibration', { 
    param: paramName, 
    value: percentage 
  }).catch((error) => {
    appendLog(`校准参数更新失败: ${error}`);
  });
}

function showSerialLog(): void {
  // 显示串口日志
  invoke('open_serial_log_window')
    .catch((error) => {
      messageService.error("打开串口日志窗口失败: " + error);
    });
}

// 处理接收到的消息
function handleMessage(message: Message): void {
  switch (message.type) {
    case 'image':
      handleImageMessage(message);
      break;
    case 'log':
      appendLog(message.data);
      break;
    case 'status':
      handleStatusMessage(message);
      break;
  }
}

// 处理图像消息
function handleImageMessage(message: ImageMessage): void {
  // 将 base64 数据转换为 data URL 格式
  const imageDataUrl = `data:image/jpeg;base64,${message.data}`;
  
  // 根据设备类型或当前页面更新对应的图像
  if (message.device === 'calibration' || currentPage.value === 'calibration') {
    calibrationImage.value = imageDataUrl;
  } else {
    cameraImage.value = imageDataUrl;
  }
}

// 处理状态消息
function handleStatusMessage(message: StatusMessage): void {
  const { data } = message;
  
  if (data.wifi !== undefined) {
    wifiStatus.value = data.wifi;
  }
  if (data.serial !== undefined) {
    serialStatus.value = data.serial;
  }
  if (data.ip !== undefined) {
    ipAddress.value = data.ip;
  }
  if (data.battery !== undefined) {
    // 可以添加电池状态显示
    appendLog(`电池电量: ${data.battery}%`);
  }
  if (data.brightness !== undefined) {
    brightness.value = data.brightness;
  }
}

// 定义对应的 TypeScript 类型
interface ImageEvent {
  type: 'image';
  data: {
    base64: string;
    device: string;
  };
}

interface StatusEvent {
  type: 'status';
  data: {
    wifi: string;
    serial: string;
    ip: string;
    battery: number;
    brightness: number;
  };
}

interface LogEvent {
  type: 'log';
  data: {
    message: string;
  };
}

type StreamEvent = ImageEvent | StatusEvent | LogEvent;

onMounted(() => {
  const onEvent = new Channel<StreamEvent>();
  
  onEvent.onmessage = (event: StreamEvent) => {
    console.log("Received event:", event);
    
    switch (event.type) {
      case 'image':
        // 构造 data URL 并显示
        const imageDataUrl = `data:image/jpeg;base64,${event.data.base64}`;
        
        if (event.data.device === 'face' || currentPage.value === 'main') {
          cameraImage.value = imageDataUrl;
        } else if (currentPage.value === 'calibration') {
          calibrationImage.value = imageDataUrl;
        }
        break;
      case 'log':
        appendLog(event.data.message);
        break;
    }
  };
  // 启动图像流
  invoke('start_face_image_stream', { onEvent })
    .then(() => {
      appendLog("图像流已启动");
    })
    .catch((error) => {
      appendLog(`启动图像流失败: ${error}`);
      messageService.error("启动图像流失败: " + error);
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
  background: none;
  border: none;
  color: var(--text-color);
  cursor: pointer;
  transition: all 0.3s ease;
}

.sub-nav-bar button:hover {
  color: var(--highlight-color);
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

.status-label {
  padding: 4px 8px;
  background-color: var(--widget-background);
  border-radius: 4px;
  font-size: 0.9rem;
}

.page-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

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
  overflow: hidden;
  border-radius: 8px;
}

.camera-view.no-image {
  color: #999;
  font-size: 0.9rem;
}

.camera-view img {
  width: 100%;
  height: 100%;
  object-fit: cover;
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
  background-color: var(--widget-background);
  color: var(--text-color);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.3s ease;
}

.action-button:hover {
  background-color: var(--highlight-color);
  color: #fff;
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
  background-color: var(--widget-background);
  color: var(--text-color);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  resize: vertical;
}

/* 滑块样式 */
.slider {
  width: 100%;
  height: 20px;
  position: relative;
  background-color: #444;
  border-radius: 10px;
  cursor: pointer;
}

.slider .track {
  position: absolute;
  height: 100%;
  background-color: var(--highlight-color);
  border-radius: 10px;
  transition: width 0.3s ease;
}

.slider .thumb {
  position: absolute;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 20px;
  height: 20px;
  background-color: #fff;
  border-radius: 50%;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  transition: left 0.3s ease;
}

/* 进度条样式 */
.progress-bar {
  width: 150px;
  height: 8px;
  background-color: #444;
  border-radius: 4px;
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  background-color: var(--highlight-color);
  transition: width 0.3s ease;
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
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.3s ease;
}

.send-button:hover {
  background-color: var(--highlight-color);
  color: #fff;
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
  border: 1px solid var(--border-color);
  border-radius: 4px;
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
  border: 1px solid var(--border-color);
  border-radius: 4px;
  flex-grow: 1;
  min-height: 40px;
  display: flex;
  align-items: center;
}

.log-section {
  width: 100%;
}

.log-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 5px;
}

.serial-log-button {
  padding: 5px 10px;
  background-color: var(--widget-background);
  color: var(--text-color);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.3s ease;
}

.serial-log-button:hover {
  background-color: var(--highlight-color);
  color: #fff;
}

.log-area {
  width: 100%;
  height: 150px;
  padding: 10px;
  resize: none;
  background-color: var(--widget-background);
  color: var(--text-color);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-family: monospace;
  font-size: 0.9rem;
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
  padding-right: 10px;
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
  font-size: 0.9rem;
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

/* 滚动条样式 */
.scroll-container::-webkit-scrollbar {
  width: 8px;
}

.scroll-container::-webkit-scrollbar-track {
  background: var(--widget-background);
  border-radius: 4px;
}

.scroll-container::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 4px;
}

.scroll-container::-webkit-scrollbar-thumb:hover {
  background: var(--highlight-color);
}
</style>