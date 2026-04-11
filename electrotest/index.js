const { TexWrapper, newTexWrapper, CpuSwapchain, wgpuInit } = require('../napi-test/napi-test.node')
const { app, BrowserWindow } = require('electron/main')
const { sharedTexture } = require('electron/common')
const { ipcMain } = require('electron')
const path = require('node:path')

async function getTexture() {
  try {
    let sc = new CpuSwapchain()
    let tex = sc.acquireNextTexture()
    let g = await wgpuInit();
    await g.fillFramebuffer();
    // g.
    // let tw = newTexWrapper()
    // let tex = sharedTexture.importSharedTexture(
    //   {
    //     'textureInfo': {
    //       'codedSize': {
    //         'height': tw.height,
    //         'width': tw.width
    //       },
    //       'pixelFormat': 'rgba',
    //       'handle': {
    //         'nativePixmap': {
    //           'modifier': '0', // linear; 875708754 is RGBA8888
    //           'planes': [
    //             {
    //               'fd': tw.fd(),
    //               'size': tw.width * tw.height * 4,
    //               'offset': 0,
    //               'stride': tw.width * 4 // TODO // according to DRM docs it is stride between rows
    //             }
    //           ],
    //           'supportsZeroCopyWebGpuImport': 'false'
    //         }
    //       }
    //     }
    //   }
    // )
    // console.log(tex.textureId);
    return tex
  } catch (err) {
    console.error("FATAL!")
    console.error(err)
    return null
  }
}

const createWindow = () => {
  const win = new BrowserWindow({
    width: 800,
    height: 600,
    webPreferences: {
      preload: path.join(__dirname, './preload.js')
    }
  })
  win.webContents.openDevTools()
  console.log(`electron main pid: ${process.pid}`)

  win.loadFile('index.html')
}

app.whenReady().then(() => {
  registerIPCHandlers()
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
function registerIPCHandlers() {
  ipcMain.handle('getTex', () => getTexture())
}

