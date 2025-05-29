<!-- FaceTrackerWindow.vue - 使用可复用滑动条组件 -->
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
            <!-- 使用可复用的滑动条组件 -->
            <DraggableSlider
              v-model="brightness"
              label="亮度调整"
              unit="%"
              :min="0"
              :max="100"
              :step="1"
              :throttle-ms="50"
              @input="handleBrightnessRealTimeUpdate"
              @change="handleBrightnessChange"
            />
            
            <DraggableSlider
              v-model="rotation"
              label="旋转角度调整"
              unit="°"
              :min="0"
              :max="360"
              :step="1"
              :throttle-ms="50"
              @input="handleRotationRealTimeUpdate"
              @change="handleRotationChange"
            />
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
  
          <div class="ip-section">
            <div class="ip-display">
              <label>IP地址：</label>
              <div class="ip-text">{{ ipAddress }}</div>
            </div>
            
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
      <div class="magnification-control">
        <label>放大倍率</label>
        <span>x1</span>
        <span>x3</span>
      </div>

      <div class="tracking-controls">
        <div class="scroll-container">
          <div class="tracking-parameters">
            <!-- 使用可复用滑动条组件进行校准参数调整 -->
            <DraggableSlider
              v-for="(param, key) in calibrationParams"
              :key="key"
              v-model="calibration[key as keyof CalibrationValues]"
              :label="param.label"
              unit="%"
              :min="0"
              :max="100"
              :step="0.1"
              :precision="1"
              :throttle-ms="50"
              @input="handleCalibrationRealTimeUpdate(key as keyof CalibrationValues, $event)"
              @change="handleCalibrationChange(key as keyof CalibrationValues, $event)"
            />
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
import { ref, reactive, onMounted } from 'vue';
import DraggableSlider from './DraggableSlider.vue'; // 导入可复用滑动条组件
import deviceService from '../functional/deviceService';
import messageService from '../functional/pop_window/messageService';
import { invoke, Channel } from '@tauri-apps/api/core';
import { StreamEvent, ImageMessage, Message, StatusMessage } from '../functional/message';
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

// 标定参数配置
const calibrationParams = {
  cheekLeft: { label: '左脸颊' },
  cheekRight: { label: '右脸颊' },
  jawOpen: { label: '下巴下移' },
  jawLeft: { label: '下巴左移' },
  jawRight: { label: '下巴右移' },
  mouthLeft: { label: '嘴左移' },
  mouthRight: { label: '嘴右移' },
  tongueOut: { label: '舌头伸出' },
  tongueUp: { label: '舌头上移' },
  tongueDown: { label: '舌头下移' },
  tongueLeft: { label: '舌头左移' },
  tongueRight: { label: '舌头右移' }
};

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
const rotation = ref<number>(0);

// 选项
const energyMode = ref<EnergyMode>('normal');
const useFilter = ref<boolean>(false);

// 日志内容
const logContent = ref<string>('');

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

// 添加日志的辅助函数
function appendLog(message: string): void {
  const timestamp = new Date().toLocaleTimeString();
  logContent.value += `[${timestamp}] ${message}\n`;
  
  setTimeout(() => {
    const logArea = document.querySelector('.log-area') as HTMLTextAreaElement;
    if (logArea) {
      logArea.scrollTop = logArea.scrollHeight;
    }
  }, 10);
}

// 亮度处理函数
function handleBrightnessRealTimeUpdate(value: number): void {
  // 实时更新过程中的处理（可选）
  console.log(`实时更新亮度: ${Math.round(value)}%`);
}

function handleBrightnessChange(value: number): void {
  appendLog(`亮度调整为: ${Math.round(value)}%`);
  invoke('set_brightness', { brightness: Math.round(value) })
    .catch((error) => {
      appendLog(`亮度调整失败: ${error}`);
    });
}

// 旋转角度处理函数
function handleRotationRealTimeUpdate(value: number): void {
  // 实时更新，提供连续旋转效果
  invoke('set_rotation', { rotation: value, deviceType: 1 })
    .catch((error) => {
      console.error(`实时旋转角度调整失败: ${error}`);
    });
}

function handleRotationChange(value: number): void {
  appendLog(`旋转角度调整为: ${Math.round(value)}°`);
  // 最终确认更新在实时更新中已经处理，这里可以添加额外的逻辑
}

// 校准参数处理函数
function handleCalibrationRealTimeUpdate(paramName: keyof CalibrationValues, value: number): void {
  // 实时更新校准参数
  invoke('update_calibration', { 
    param: paramName, 
    value: value 
  }).catch((error) => {
    console.error(`实时校准参数更新失败: ${error}`);
  });
}

function handleCalibrationChange(paramName: keyof CalibrationValues, value: number): void {
  appendLog(`${calibrationParams[paramName].label} 调整为: ${Math.round(value)}%`);
  // 最终确认更新在实时更新中已经处理
}

// 其他功能函数
function sendWifiSettings(): void {
  if (!ssid.value || !password.value) {
    messageService.warning("请输入WIFI名称和密码");
    return;
  }
  
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

function showSerialLog(): void {
  invoke('open_serial_log_window')
    .catch((error) => {
      messageService.error("打开串口日志窗口失败: " + error);
    });
}

onMounted(() => {
  const onImageOrLogEvent = new Channel<StreamEvent>();
  
  onImageOrLogEvent.onmessage = (event: StreamEvent) => {
    switch (event.type) {
      case 'image':
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

  invoke('start_face_image_stream', { onEvent: onImageOrLogEvent })
    .then(() => {
      appendLog("图像流已启动");
    })
    .catch((error) => {
      appendLog(`启动图像流失败: ${error}`);
      messageService.error("启动图像流失败: " + error);
    });

  listen<string>('face_serial_status', (event) => {
      serialStatus.value = event.payload;
  });

  listen<string>('face_image_stream_status', (event) => {
      wifiStatus.value = event.payload;
  }) 
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

.adjustments {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.option-controls {
  display: flex;
  align-items: center;
  gap: 20px;
  margin-top: 10px;
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

.ip-section {
  display: flex;
  flex-direction: column;
  gap: 15px;
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
  gap: 15px;
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