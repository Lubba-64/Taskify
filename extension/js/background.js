import init, { WebHandle } from "./wasm/taskify_extension_background.js";
const wasmUrl = new URL("./wasm/taskify_extension_background_bg.wasm", import.meta.url);
await init({ module_or_path: wasmUrl });
const handle = new WebHandle();
console.debug("background initialized");
setInterval(() => handle.tick(), 100);
browser.runtime.onMessage.addListener(data => {
    const messageType = data?.type ?? data?.data;
    if (messageType == 'scan_text'){
        console.log("textscan");
        handle.scan_text();
    }
    if (messageType == 'scan_pdf'){
        handle.scan_pdf();
    }
    if (messageType == 'scan_image'){
        // TODO: IMPL image
    }
})
