const x = require('napi-test')
const { app, BrowserWindow } = require('electron/main')
const {sharedTexture} = require('electron/common')

console.log(x.plus100(12))

const createWindow = () => {
  const win = new BrowserWindow({
    width: 800,
    height: 600,
  })

  sharedTexture.importSharedTexture()
    
  win.loadFile('index.html')
}

app.whenReady().then(() => {
  createWindow()

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow()
    }
  })
})

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit()
  }
})
