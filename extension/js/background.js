import init, { WebHandle } from "./wasm/taskify_extension_background.js";
const wasmUrl = new URL("./wasm/taskify_extension_background_bg.wasm", import.meta.url);
await init({ module_or_path: wasmUrl });
const handle = new WebHandle();
console.debug("background initialized");
setInterval(() => handle.tick(), 100);

 browser.windows.create( {
      type: 'popup',
      width: 640,
      height: 480,
      url: 'popup.htm'
    } );

browser.runtime.onMessage.addListener(data => {
    const messageType = data?.type ?? data?.data;
    if (messageType == 'scan_text'){
        console.log("textscan");
        try {
            handle.scan_text();
        } catch (err) {
            console.error("scan_text threw", err);
        }
    }
    if (messageType == 'scan_pdf'){
        try {
            handle.scan_pdf();
        } catch (err) {
            console.error("scan_pdf threw", err);
        }
    }
    if (messageType == 'scan_image'){
        handle.scan_image();
    }
});
