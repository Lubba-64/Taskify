import init, { WebHandle } from "./wasm/taskify_extension.js";

window.addEventListener('blur', (e) => {
  // Try to keep focus
  window.focus();
});

function showCrash(handle) {
    document.getElementById("the_canvas_id")?.remove();
    document.getElementById("center_text").innerHTML = `
    <p>
        The egui app has crashed.
    </p>
    <p style="font-size:10px" align="left">
        ${handle.panic_message()}
    </p>
    <p style="font-size:14px">
        See the console for details.
    </p>
    <p style="font-size:14px">
        Reload the page to try again.
    </p>`;
}

function showError(error) {
    console.error("Failed to start:", error);
    document.getElementById("the_canvas_id")?.remove();
    document.getElementById("center_text").innerHTML = `
    <p>
        An error occurred during loading:
    </p>
    <p style="font-family:Courier New">
        ${error}
    </p>
    <p style="font-size:14px">
        Make sure you use a modern browser with WebGL and WASM enabled.
    </p>`;
}

function onAppStarted() {
    console.debug("App started.");
    document.getElementById("center_text").innerHTML = "";
    document.getElementById("the_canvas_id")?.focus();
}

function startPanicPolling(handle) {
    function checkForPanic() {
        if (handle.has_panicked()) {
            console.error("The egui app has crashed");
            showCrash(handle);
            return;
        }

        setTimeout(checkForPanic, 1000);
    }

    checkForPanic();
}

async function bootstrap() {
    console.debug("Loading wasm...");

    const wasmUrl = new URL("./wasm/taskify_extension_bg.wasm", import.meta.url);
    await init({ module_or_path: wasmUrl });
    console.debug("Wasm loaded. Starting app...");

    const handle = new WebHandle();
    startPanicPolling(handle);

    const canvas = document.getElementById("the_canvas_id");

    canvas.addEventListener("webglcontextlost", (e) => {
        e.preventDefault();
        console.warn("WebGL context lost, destroying handle...");
        handle.destroy();
    }, false);

    canvas.addEventListener("webglcontextrestored", () => {
        console.warn("WebGL context restored, restarting...");
        bootstrap().catch(showError);
    }, false);

    await handle.start(canvas);
    onAppStarted();
}

bootstrap().catch(showError);
