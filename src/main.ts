import { createApp } from "vue";
import App from "./App.vue";
import "./assets/style.css";
import Modal from "./components/pop_window/Modal.vue";
import MessageBox from "./components/pop_window/MessageBox.vue";
import ConfirmBox from "./components/pop_window/ConfirmBox.vue";

// 创建Vue应用实例
const app = createApp(App);

// 注册全局组件
app.component('Modal', Modal);
app.component('MessageBox', MessageBox);
app.component('ConfirmBox', ConfirmBox);

// 挂载应用
app.mount("#app");

// 全局消息服务
import messageService from './functional/pop_window/messageService';
import confirmService from './functional/pop_window/confirmService';

// 为了方便在控制台调试，扩展 Window 接口
declare global {
  interface Window {
    messageService: typeof messageService;
    confirmService: typeof confirmService;
  }
}

window.messageService = messageService;
window.confirmService = confirmService;