let eapi = window.electronAPI
let x = eapi.acquireTexture().then(t => console.log(t));
document.getElementById("spam").innerText = "x";

