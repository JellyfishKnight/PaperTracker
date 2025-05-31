// 定义对应的 TypeScript 类型
export interface ImageEvent {
    type: 'image';
    data: {
        base64: string;
        device: string;
    };
}
  
export interface StatusEvent {
    type: 'status';
    data: {
        ip: string;
        serial: boolean;
        battery: number;
        brightness: number;
        device_type: number; // 设备类型
    };
}

export interface LogEvent {
    type: 'log';
    data: {
        message: string;
    };
}
  
export type StreamEvent = ImageEvent | StatusEvent | LogEvent;

// 定义消息类型
export interface ImageMessage {
    type: 'image';
    data: string;  // base64 图像数据
    device?: string;  // 可选：设备标识
}
  
export interface LogMessage {
    type: 'log';
    data: string;
}
  
export interface StatusMessage {
    type: 'status';
    data: {
        wifi?: string;
        serial?: boolean;
        battery?: number;
        brightness?: number;
        deviceType?: number;  // 设备类型
    };
}
  
export type Message = ImageMessage | LogMessage | StatusMessage;
  