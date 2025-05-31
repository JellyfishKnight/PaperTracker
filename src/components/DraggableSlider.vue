<!-- components/DraggableSlider.vue -->
<!-- 
可复用的拖拽滑动条组件
特性：
1. 完全可拖动，支持点击定位和鼠标拖拽
2. 实时更新回调，支持拖拽过程中的连续反馈
3. 可配置的节流机制
4. 视觉反馈和状态指示
5. 完整的参数配置支持
-->
<template>
    <div class="slider-group">
      <label v-if="label" class="slider-label">{{ label }}</label>
      
      <div 
        class="draggable-slider" 
        :class="{ disabled: disabled }"
        ref="sliderElement"
        @mousedown="startDrag"
        @click="handleClick"
      >
        <div class="slider-track" :style="{ width: percentage + '%' }"></div>
        <div 
          class="slider-thumb" 
          :style="{ left: percentage + '%' }"
          :class="{ dragging: isDragging }"
        ></div>
      </div>
      
      <span class="slider-value">
        {{ displayValue }}{{ unit }}
        <span v-if="isDragging && showRealTimeIndicator" class="realtime-indicator">●</span>
      </span>
      
      <!-- 调试信息 -->
      <div v-if="debug" class="debug-info">
        <div>节流: {{ throttleMs }}ms</div>
        <div>拖拽: {{ isDragging ? '是' : '否' }}</div>
        <div>待更新: {{ pendingUpdate ? '是' : '否' }}</div>
        <div>上次更新: {{ lastUpdateTime > 0 ? `${Date.now() - lastUpdateTime}ms前` : '无' }}</div>
      </div>
    </div>
  </template>
  
  <script setup lang="ts">
  import { ref, computed, onUnmounted, watch } from 'vue';
  
  interface SliderProps {
    modelValue: number;
    min?: number;
    max?: number;
    step?: number;
    disabled?: boolean;
    label?: string;
    unit?: string;
    throttleMs?: number;
    showRealTimeIndicator?: boolean;
    precision?: number; // 显示精度
    debug?: boolean; // 调试模式
  }
  
  interface SliderEmits {
    'update:modelValue': [value: number];
    'change': [value: number]; // 最终变化（拖拽结束或点击）
    'input': [value: number];  // 实时变化（拖拽过程中）
  }
  
  const props = withDefaults(defineProps<SliderProps>(), {
    min: 0,
    max: 100,
    step: 1,
    disabled: false,
    label: '',
    unit: '',
    throttleMs: 50,
    showRealTimeIndicator: true,
    precision: 0,
    debug: false
  });
  
  const emit = defineEmits<SliderEmits>();
  
  // 组件状态
  const sliderElement = ref<HTMLElement | null>(null);
  const isDragging = ref<boolean>(false);
  const lastUpdateTime = ref<number>(0);
  const pendingUpdate = ref<boolean>(false);
  const throttleTimer = ref<number | null>(null);
  
  // 计算属性
  const percentage = computed(() => {
    const range = props.max - props.min;
    if (range === 0) return 0;
    return ((props.modelValue - props.min) / range) * 100;
  });
  
  const displayValue = computed(() => {
    if (props.precision === 0) {
      return Math.round(props.modelValue);
    }
    return props.modelValue.toFixed(props.precision);
  });
  
  // 工具函数
  function getValueFromPosition(clientX: number): number {
    if (!sliderElement.value || props.disabled) return props.modelValue;
    
    const rect = sliderElement.value.getBoundingClientRect();
    const x = clientX - rect.left;
    const percentage = Math.min(100, Math.max(0, (x / rect.width) * 100));
    
    const range = props.max - props.min;
    let value = props.min + (percentage / 100) * range;
    
    // 应用步长
    if (props.step > 0) {
      value = Math.round(value / props.step) * props.step;
    }
    
    // 确保值在范围内
    return Math.min(props.max, Math.max(props.min, value));
  }
  
  function updateValue(clientX: number, isRealTime: boolean = false): void {
    if (props.disabled) return;
    
    const newValue = getValueFromPosition(clientX);
    if (newValue === props.modelValue) return;
    
    emit('update:modelValue', newValue);
    
    if (isRealTime) {
      // 实时更新时使用节流
      throttledEmitInput(newValue);
    } else {
      // 最终更新时直接emit
      emit('change', newValue);
    }
  }
  
  // 修正的节流函数 - 真正实现节流功能
  function throttledEmitInput(value: number): void {
    const now = Date.now();
    
    // 检查是否在节流时间内
    if (now - lastUpdateTime.value < props.throttleMs) {
      // 如果还没有待处理的更新，设置一个延时更新
      if (!pendingUpdate.value) {
        pendingUpdate.value = true;
        
        // 清除之前的定时器
        if (throttleTimer.value !== null) {
          clearTimeout(throttleTimer.value);
        }
        
        // 设置新的定时器
        throttleTimer.value = window.setTimeout(() => {
          if (pendingUpdate.value && isDragging.value) {
            lastUpdateTime.value = Date.now();
            emit('input', props.modelValue);
            console.log(`节流更新: ${props.modelValue} (延迟: ${Date.now() - now}ms)`);
          }
          pendingUpdate.value = false;
          throttleTimer.value = null;
        }, props.throttleMs - (now - lastUpdateTime.value));
      }
      return;
    }
    
    // 如果超过节流时间，立即更新
    lastUpdateTime.value = now;
    pendingUpdate.value = false;
    
    // 清除待处理的定时器
    if (throttleTimer.value !== null) {
      clearTimeout(throttleTimer.value);
      throttleTimer.value = null;
    }
    
    emit('input', value);
    console.log(`立即更新: ${value}`);
  }
  
  // 事件处理
  function handleClick(event: MouseEvent): void {
    if (props.disabled || isDragging.value) return;
    
    // 如果点击的是滑块本身，不处理（让拖动处理）
    if ((event.target as HTMLElement).classList.contains('slider-thumb')) {
      return;
    }
    
    updateValue(event.clientX, false);
  }
  
  function startDrag(event: MouseEvent): void {
    if (props.disabled) return;
    
    event.preventDefault();
    isDragging.value = true;
    
    // 立即更新到点击位置
    updateValue(event.clientX, true);
    
    // 添加全局事件监听器
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
    document.addEventListener('selectstart', preventDefault);
  }
  
  function handleMouseMove(event: MouseEvent): void {
    if (!isDragging.value || props.disabled) return;
    
    event.preventDefault();
    updateValue(event.clientX, true);
  }
  
  function handleMouseUp(event: MouseEvent): void {
    if (!isDragging.value) return;
    
    // 最终更新
    updateValue(event.clientX, false);
    
    // 清理状态
    isDragging.value = false;
    pendingUpdate.value = false;
    
    // 清理定时器
    if (throttleTimer.value !== null) {
      clearTimeout(throttleTimer.value);
      throttleTimer.value = null;
    }
    
    // 移除全局事件监听器
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
    document.removeEventListener('selectstart', preventDefault);
  }
  
  function preventDefault(event: Event): void {
    event.preventDefault();
  }
  
  // 监听外部值变化
  watch(() => props.modelValue, (newValue) => {
    // 确保值在有效范围内
    const clampedValue = Math.min(props.max, Math.max(props.min, newValue));
    if (clampedValue !== newValue) {
      emit('update:modelValue', clampedValue);
    }
  });
  
  // 组件卸载时清理
  onUnmounted(() => {
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
    document.removeEventListener('selectstart', preventDefault);
    
    // 清理定时器
    if (throttleTimer.value !== null) {
      clearTimeout(throttleTimer.value);
      throttleTimer.value = null;
    }
    
    isDragging.value = false;
    pendingUpdate.value = false;
  });
  </script>
  
  <style scoped>
  .slider-group {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 32px;
    position: relative; /* 为调试信息提供定位上下文 */
  }
  
  .slider-label {
    min-width: 100px;
    font-size: 0.9rem;
    color: var(--text-color, #a9b7c6);
  }
  
  .draggable-slider {
    width: 200px;
    height: 20px;
    position: relative;
    background-color: #444;
    border-radius: 10px;
    cursor: pointer;
    user-select: none;
    transition: opacity 0.3s ease;
  }
  
  .draggable-slider.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .slider-track {
    position: absolute;
    height: 100%;
    background-color: var(--highlight-color, #04b97f);
    border-radius: 10px;
    transition: width 0.1s ease;
    pointer-events: none;
  }
  
  .slider-thumb {
    position: absolute;
    top: 50%;
    transform: translate(-50%, -50%);
    width: 20px;
    height: 20px;
    background-color: #fff;
    border-radius: 50%;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
    cursor: grab;
    transition: left 0.1s ease, transform 0.2s ease, box-shadow 0.2s ease;
    pointer-events: none;
  }
  
  .slider-thumb.dragging {
    cursor: grabbing;
    transition: none;
    box-shadow: 0 4px 12px rgba(4, 185, 127, 0.5);
    transform: translate(-50%, -50%) scale(1.1);
  }
  
  .draggable-slider:hover .slider-thumb:not(.dragging) {
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
    transform: translate(-50%, -50%) scale(1.05);
  }
  
  .draggable-slider.disabled .slider-thumb {
    cursor: not-allowed;
    background-color: #999;
  }
  
  .slider-value {
    min-width: 60px;
    text-align: right;
    font-size: 0.9rem;
    color: var(--highlight-color, #04b97f);
    font-weight: bold;
    display: flex;
    align-items: center;
    gap: 5px;
  }
  
  .realtime-indicator {
    color: var(--highlight-color, #04b97f);
    animation: pulse 1s infinite;
    font-size: 0.6rem;
  }
  
  @keyframes pulse {
    0% { opacity: 1; }
    50% { opacity: 0.5; }
    100% { opacity: 1; }
  }
  
  /* 调试信息样式 */
  .debug-info {
    position: absolute;
    top: 100%;
    left: 0;
    background: rgba(0, 0, 0, 0.8);
    color: white;
    padding: 5px 8px;
    border-radius: 4px;
    font-size: 0.7rem;
    white-space: nowrap;
    z-index: 1000;
    margin-top: 5px;
  }
  
  .debug-info div {
    margin: 1px 0;
  }
  
  /* 响应式设计 */
  @media (max-width: 768px) {
    .slider-group {
      flex-direction: column;
      align-items: flex-start;
      gap: 5px;
    }
    
    .slider-label {
      min-width: auto;
      width: 100%;
    }
    
    .draggable-slider {
      width: 100%;
      min-width: 200px;
    }
    
    .slider-value {
      align-self: flex-end;
    }
  }
  </style>