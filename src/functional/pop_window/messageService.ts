// utils/messageService.ts
import { createApp, h } from 'vue';
import MessageBox from '../../components/pop_window/MessageBox.vue';
import { MessageService, MessageOptions } from '../types';

// 创建容器元素
function createContainer(): HTMLElement {
  const container = document.createElement('div');
  container.className = 'message-container';
  document.body.appendChild(container);
  return container;
}

// 消息服务实例
const messageService: MessageService = {
  // 消息容器
  container: null,
  // 消息实例引用
  instance: null,
  
  // 初始化
  init() {
    if (!this.container) {
      this.container = createContainer();
      const app = createApp({
        render() {
          return h(MessageBox, { ref: 'messageBox' });
        }
      });
      
      const vm = app.mount(this.container);
      this.instance = vm.$refs.messageBox;
    }
    return this.instance;
  },
  
  // 显示消息
  show(options: MessageOptions = { message: '' }) {
    const instance = this.init();
    instance.open(options);
    return instance;
  },
  
  // 成功消息
  success(message: string, title: string = '成功', callback?: () => void) {
    return this.show({
      type: 'success',
      message,
      title,
      callback
    });
  },
  
  // 错误消息
  error(message: string, title: string = '错误', callback?: () => void) {
    return this.show({
      type: 'error',
      message,
      title,
      callback
    });
  },
  
  // 警告消息
  warning(message: string, title: string = '警告', callback?: () => void) {
    return this.show({
      type: 'warning',
      message,
      title,
      callback
    });
  },
  
  // 信息消息
  info(message: string, title: string = '信息', callback?: () => void) {
    return this.show({
      type: 'info',
      message,
      title,
      callback
    });
  }
};

export default messageService;