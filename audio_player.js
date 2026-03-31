/**
 * 动态波形播放器 - 使用AudioWorklet播放以60fps为单位追加的波形
 * 对外暴露 appendWaveform 方法
 */

const DynamicWaveformPlayer = (function() {
    let audioContext = null;
    let workletNode = null;
    let initialized = false;
    let isPlaying = false;
    let sampleRate = 48000;
    
    const workletProcessorCode = `
        class WaveformProcessor extends AudioWorkletProcessor {
            constructor() {
                super();
                this.waveformBuffer = new Float32Array(0);
                this.currentPosition = 0;
                
                this.port.onmessage = (event) => {
                    if (event.data.type === 'append') {
                        this.appendWaveform(event.data.waveform);
                    } else if (event.data.type === 'clear') {
                        this.clearBuffer();
                    }
                };
            }
            
            appendWaveform(newWaveform) {
                const newData = new Float32Array(newWaveform);
                const combinedBuffer = new Float32Array(this.waveformBuffer.length + newData.length);
                combinedBuffer.set(this.waveformBuffer, 0);
                combinedBuffer.set(newData, this.waveformBuffer.length);
                this.waveformBuffer = combinedBuffer;
            }
            
            process(inputs, outputs, parameters) {
                const output = outputs[0];
                const outputChannel = output[0];
                
                if (this.waveformBuffer.length === 0 || this.currentPosition >= this.waveformBuffer.length) {
                    //outputChannel.fill(0);
                    return true;
                }
                
                //this.bufferLock = true;
                
                const samplesToCopy = Math.min(
                    outputChannel.length,
                    this.waveformBuffer.length - this.currentPosition
                );
                
                for (let i = 0; i < samplesToCopy; i++) {
                    outputChannel[i] = this.waveformBuffer[this.currentPosition + i];
                }
                
                for (let i = samplesToCopy; i < outputChannel.length; i++) {
                    outputChannel[i] = 0;
                }
                
                this.currentPosition += samplesToCopy;

                const maxLen = 50000
                if (this.currentPosition >= maxLen) {
                    this.waveformBuffer = this.waveformBuffer.slice(maxLen);
                    this.currentPosition -= maxLen;
                }
                
                return true;
            }
        }
        
        registerProcessor('waveform-processor', WaveformProcessor);
    `;
    
    // 初始化AudioContext和Worklet
    async function initialize() {
        if (initialized) return true;
        
        try {
            audioContext = new (window.AudioContext || window.webkitAudioContext)();
            sampleRate = audioContext.sampleRate;
            
            const blob = new Blob([workletProcessorCode], { type: 'application/javascript' });
            const blobURL = URL.createObjectURL(blob);
            
            await audioContext.audioWorklet.addModule(blobURL);
            
            workletNode = new AudioWorkletNode(audioContext, 'waveform-processor');
            
            workletNode.connect(audioContext.destination);
            
            URL.revokeObjectURL(blobURL);
            
            initialized = true;
            // console.log('DynamicWaveformPlayer 初始化成功，采样率:', sampleRate);
            
            return true;
        } catch (error) {
            console.error('初始化失败:', error);
            return false;
        }
    }
    
    async function ensureAudioContextRunning() {
        if (audioContext && audioContext.state !== 'running') {
            await audioContext.resume();
        }
    }
    
    // 公共API
    return {
        /**
         * 初始化播放器
         * @returns {Promise<boolean>} 初始化是否成功
         */
        async init() {
            return await initialize();
        },
        
        /**
         * 开始播放
         * @returns {Promise<boolean>} 是否成功开始播放
         */
        async play() {
            if (!initialized) {
                const success = await this.init();
                if (!success) return false;
            }
            
            try {
                await ensureAudioContextRunning();
                isPlaying = true;
                return true;
            } catch (error) {
                console.error('播放失败:', error);
                return false;
            }
        },
        
        /**
         * 追加波形数据
         * @param {Float32Array|Array<number>} waveform 波形数据（值范围应在-1到1之间）
         * @param {number} durationInFrames 持续时间（帧数，1/60秒为单位），如果提供此参数，将自动生成对应长度的正弦波（仅用于演示）
         * @returns {boolean} 是否成功追加
         */
        appendWaveform(waveform) {
            if (!initialized) {
                console.warn('播放器未初始化，请先调用init()');
                return false;
            }
            
            if (!workletNode) return false;
            
            try {
                let waveformData;
                
                if (waveform) {
                    if (waveform instanceof Float32Array) {
                        waveformData = waveform;
                    } else if (Array.isArray(waveform)) {
                        waveformData = new Float32Array(waveform);
                    } else {
                        throw new Error('waveform必须是Float32Array或数组');
                    }
                } else {
                    throw new Error('必须提供waveform');
                }
                
                // 发送数据到AudioWorklet
                workletNode.port.postMessage({
                    type: 'append',
                    waveform: waveformData
                });
                
                return true;
            } catch (error) {
                console.error('追加波形失败:', error);
                return false;
            }
        },
        
        /**
         * 获取当前播放状态
         */
        get isPlaying() {
            return isPlaying && audioContext && audioContext.state === 'running';
        },
        
        /**
         * 获取采样率
         */
        getSampleRate() {
            return sampleRate;
        },
        
        /**
         * 关闭播放器
         */
        close() {
            if (audioContext) {
                audioContext.close();
                audioContext = null;
            }
            workletNode = null;
            initialized = false;
            isPlaying = false;
        }
    };
})();

// 导出模块（适用于CommonJS和ES6模块环境）
if (typeof module !== 'undefined' && module.exports) {
    module.exports = DynamicWaveformPlayer;
} else if (typeof define === 'function' && define.amd) {
    define([], () => DynamicWaveformPlayer);
} else {
    window.DynamicWaveformPlayer = DynamicWaveformPlayer;
}

// 默认导出（适用于ES6模块）
export default DynamicWaveformPlayer;
