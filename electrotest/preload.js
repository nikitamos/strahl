const { contextBridge, ipcRenderer } = require('electron/renderer');

console.log('PRELOAD')
contextBridge.exposeInMainWorld(
    'electronAPI', {
    acquireTexture: () => ipcRenderer.invoke('getTex')
}
)