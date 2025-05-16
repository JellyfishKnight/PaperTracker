// functional/device_operations/deviceService.ts (updated)
import { createApp, h } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

import ProgressBox from '../components/pop_window/ProgressBox.vue';
import FirmwareSelector from '../components/pop_window/FirmwareSelector.vue';
import messageService from './pop_window/messageService';

// 固件类型定义
export interface FirmwareSelection {
  deviceType: 'face' | 'left_eye' | 'right_eye' | 'custom';
  firmwareType: 'stable' | 'beta' | 'custom';
  firmwarePath: string | null;
}

// 进度更新结构体
interface ProgressUpdate {
  progress: number;
  message: string;
  status: string;
}

// 创建容器元素
function createProgressContainer(): HTMLElement {
  const container = document.createElement('div');
  container.className = 'progress-container';
  document.body.appendChild(container);
  return container;
}

// 创建固件选择器容器
function createSelectorContainer(): HTMLElement {
  const container = document.createElement('div');
  container.className = 'selector-container';
  document.body.appendChild(container);
  return container;
}

// 设备操作服务
const deviceService = {
  // 容器
  progressContainer: null as HTMLElement | null,
  selectorContainer: null as HTMLElement | null,
  
  // 实例引用
  progressInstance: null as any,
  selectorInstance: null as any,
  
  // 进度控制器
  progressControl: null as any,
  
  // 事件监听器
  eventUnlisten: null as (() => void) | null,
  
  // 初始化进度框
  initProgressBox() {
    if (!this.progressContainer) {
      this.progressContainer = createProgressContainer();
      const app = createApp({
        render() {
          return h(ProgressBox, { ref: 'progressBox' });
        }
      });
      
      const vm = app.mount(this.progressContainer);
      this.progressInstance = vm.$refs.progressBox;
    }
    return this.progressInstance;
  },
  
  // 初始化固件选择器
  initFirmwareSelector() {
    if (!this.selectorContainer) {
      this.selectorContainer = createSelectorContainer();
      const app = createApp({
        render() {
          return h(FirmwareSelector, { ref: 'firmwareSelector' });
        }
      });
      
      const vm = app.mount(this.selectorContainer);
      this.selectorInstance = vm.$refs.firmwareSelector;
    }
    return this.selectorInstance;
  },
  
  // 设置事件监听
  async setupEventListener() {
    if (this.eventUnlisten) {
      // 已经设置过监听器，无需重复设置
      return;
    }
    
    // 监听ESP32操作进度事件
    this.eventUnlisten = await listen<ProgressUpdate>('esp32_operation', (event) => {
      const { progress, message, status } = event.payload;
      
      if (!this.progressControl) {
        return;
      }
      
      // 更新进度
      this.progressControl.updateProgress(progress);
      
      // 如果有消息，更新消息
      if (message) {
        // 更新进度框消息
        const progressElement = document.querySelector('.progress-message');
        if (progressElement) {
          progressElement.textContent = message;
        }
      }
      
      // 如果操作完成
      if (progress >= 100 || status === 'error' || status === 'success') {
        if (status === 'error') {
          // 显示错误消息
          messageService.error(message, '操作失败');
        } else if (status === 'success') {
          // 显示成功消息
          messageService.success(message, '操作完成');
        }
      }
    });
  },
  
  // 重启 ESP32 设备
  async restartESP32() {
    // 设置事件监听
    await this.setupEventListener();
    
    // 初始化进度框
    const progress = this.initProgressBox();
    
    // 打开进度框
    this.progressControl = progress.open({
      title: '正在重启ESP32设备',
      message: '正在准备重启设备，请稍候...',
      cancelable: true,
      initialProgress: 0,
      // 自动进度更新函数
      autoProgress: (current: number) => {
        if (current < 85) {
          return current + 0.5; // 缓慢增加进度
        }
        return current;
      },
      onComplete: () => {
        console.log('重启操作完成');
      },
      onCancel: () => {
        console.log('用户取消了重启操作');
      }
    });
    
    try {
      // 调用后端重启函数
      await invoke('restart_esp32');
    } catch (error) {
      console.error('调用重启ESP32函数失败:', error);
      
      // 显示错误消息
      messageService.error(`调用重启函数失败: ${error}`, '操作失败');
      
      // 关闭进度框
      if (this.progressControl) {
        this.progressControl.cancel();
        this.progressControl = null;
      }
    }
  },
  
  // 刷写ESP32固件
  async flashESP32() {
    // 初始化固件选择器
    const selector = this.initFirmwareSelector();
    
    // 打开固件选择器
    selector.open({
      title: '选择固件',
      description: '请选择要刷写的设备类型和固件版本',
      stableVersion: 'v1.2.1',
      betaVersion: 'v1.3.0-beta',
      onConfirm: async (selection: FirmwareSelection) => {
        // 用户已确认选择，开始刷写过程
        console.log('用户选择:', selection);
        
        // 设置事件监听
        await this.setupEventListener();
        
        // 初始化进度框
        const progress = this.initProgressBox();
        
        // 打开进度框
        this.progressControl = progress.open({
          title: '正在刷写ESP32固件',
          message: `正在刷写${this.getDeviceTypeName(selection.deviceType)}固件，请不要断开设备连接...`,
          cancelable: false, // 刷写过程不可取消
          initialProgress: 0,
          // 自动进度更新函数
          autoProgress: (current: number) => {
            if (current < 95) {
              // 模拟刷写进度
              return current + 0.2;
            }
            return current;
          },
          onComplete: () => {
            console.log('刷写操作完成');
          }
        });
        
        try {
          // 准备参数
          const params: any = {
            device_type: selection.deviceType,
            firmware_type: selection.firmwareType
          };
          
          // 如果是自定义固件，添加路径
          if (selection.firmwareType === 'custom' && selection.firmwarePath) {
            params.firmware_path = selection.firmwarePath;
          }
          
          // 调用后端刷写函数
          await invoke('flash_esp32', params);
        } catch (error) {
          console.error('调用刷写ESP32固件函数失败:', error);
          
          // 显示错误消息
          messageService.error(`调用刷写函数失败: ${error}`, '操作失败');
          
          // 关闭进度框
          if (this.progressControl) {
            this.progressControl.cancel();
            this.progressControl = null;
          }
        }
      },
      onCancel: () => {
        console.log('用户取消了固件选择');
      }
    });
  },
  
  // 获取设备类型的显示名称
  getDeviceTypeName(deviceType: string): string {
    switch (deviceType) {
      case 'face':
        return '面捕';
      case 'left_eye':
        return '左眼';
      case 'right_eye':
        return '右眼';
      default:
        return deviceType;
    }
  }
};

export default deviceService;