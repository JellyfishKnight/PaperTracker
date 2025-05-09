// utils/types.ts

// 消息框配置接口
export interface MessageOptions {
    message: string;
    title?: string;
    type?: 'default' | 'success' | 'error' | 'warning' | 'info';
    buttonText?: string;
    callback?: () => void;
  }
  
  // 确认框配置接口
  export interface ConfirmOptions {
    message: string;
    title?: string;
    confirmText?: string;
    cancelText?: string;
    showIcon?: boolean;
    onConfirm?: () => void;
    onCancel?: () => void;
  }
  
  // 消息服务接口
  export interface MessageService {
    container: HTMLElement | null;
    instance: any | null;
    init(): any;
    show(options: MessageOptions): any;
    success(message: string, title?: string, callback?: () => void): any;
    error(message: string, title?: string, callback?: () => void): any;
    warning(message: string, title?: string, callback?: () => void): any;
    info(message: string, title?: string, callback?: () => void): any;
  }
  
  // 确认服务接口
  export interface ConfirmService {
    container: HTMLElement | null;
    instance: any | null;
    init(): any;
    confirm(message: string, title?: string, onConfirm?: () => void, onCancel?: () => void): any;
    danger(message: string, title?: string, onConfirm?: () => void, onCancel?: () => void): any;
  }