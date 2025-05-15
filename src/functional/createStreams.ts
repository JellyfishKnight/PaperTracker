// src/functional/cameraStreams.ts
import { invoke, Channel } from '@tauri-apps/api/core';

type UnlistenFn = () => void;
    export interface CameraStreamManager {
    startFaceStream: () => Promise<void>;
    
    startLeftEyeStream: () => Promise<void>;
    
    startRightEyeStream: () => Promise<void>;
}

const cameraStreamManager: CameraStreamManager = {
    // Face camera
    async startFaceStream() {
        try {
            await invoke('start_face_stream', {  });
            console.log('Face camera stream started');
        } catch (error) {
            console.error('Failed to start face camera stream:', error);
            throw error;
        }
    },


    // Left eye camera
    async startLeftEyeStream() {
        try {
            await invoke('start_left_eye_stream');
            console.log('Left eye camera stream started');
        } catch (error) {
            console.error('Failed to start left eye camera stream:', error);
            throw error;
        }
    },

    // Right eye camera
    async startRightEyeStream() {
        try {
            await invoke('start_right_eye_stream');
            console.log('Right eye camera stream started');
        } catch (error) {
            console.error('Failed to start right eye camera stream:', error);
            throw error;
        }
    },

};

export default cameraStreamManager;