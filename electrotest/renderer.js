let eapi = window.electronAPI
doStuff().then(() => { })
document.getElementById("spam").innerText = "x";

async function doStuff() {
    let tex = new Uint8ClampedArray((await eapi.acquireTexture()).buffer);
    const width = 20;
    const height = tex.length / (width * 4);

    let bm = await createImageBitmap(new ImageData(tex, width, height));

    const canvas = document.getElementById('myCanvas'); // Make sure you have a canvas element
    const ctx = canvas.getContext('2d');

    canvas.width = width;
    canvas.height = height;

    // Draw the image bitmap
    ctx.drawImage(bm, 0, 0)
}

