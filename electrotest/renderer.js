function rx(data) {
    console.log("Shared texture received!");
    let frame = data.importedSharedTexture.getVideoFrame();
    const canvas = document.getElementById('spam');
    const ctx = canvas.getContext('2d');

    ctx.drawImage(frame, 0, 0);
}

window.electronAPI.setSharedTextureReceiver(rx).then(
    () => {
        let x = window.electronAPI.acquireTexture().then(t => console.log(t));
    })
// document.getElementById("spam").innerText = "x";


