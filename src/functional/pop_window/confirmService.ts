// utils/confirmService.ts
import { createApp, h } from 'vue';
import ConfirmBox from '../../components/pop_window/ConfirmBox.vue';
import { ConfirmService } from '../types';

// 创建容器元素
function createContainer(): HTMLElement {
  const container = document.createElement('div');
  container.className = 'confirm-container';
  document.body.appendChild(container);
  return container;
}

// 确认对话框服务实例
const confirmService: ConfirmService = {
  // 容器
  container: null,
  // 实例引用
  instance: null,
  
  // 初始化
  init() {
    if (!this.container) {
      this.container = createContainer();
      const app = createApp({
        render() {
          return h(ConfirmBox, { ref: 'confirmBox' });
        }
      });
      
      const vm = app.mount(this.container);
      this.instance = vm.$refs.confirmBox;
    }
    return this.instance;
  },
  
  // 显示确认对话框
  confirm(message: string, title: string = '确认', onConfirm?: () => void, onCancel?: () => void) {
    const instance = this.init();
    instance.open({
      message,
      title,
      onConfirm,
      onCancel
    });
    return instance;
  },
  
  // 危险操作确认
  danger(message: string, title: string = '警告', onConfirm?: () => void, onCancel?: () => void) {
    const instance = this.init();
    instance.open({
      message,
      title,
      confirmText: '确认删除',
      onConfirm,
      onCancel
    });
    return instance;
  }
};

export default confirmService;