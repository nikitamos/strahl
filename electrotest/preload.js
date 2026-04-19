const { contextBridge, ipcRenderer } = require('electron/renderer');
const { sharedTexture } = require('electron');

console.log('PRELOAD')
contextBridge.exposeInMainWorld(
    'electronAPI', {
        acquireTexture: () => ipcRenderer.invoke('getTex'),
        setSharedTextureReceiver: async (cbk) => sharedTexture.setSharedTextureReceiver(cbk)
}
)