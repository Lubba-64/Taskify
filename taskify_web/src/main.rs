use axum::response::Html;
use axum::{routing::get, Router};
use dotenv::dotenv;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let app = Router::new()
        .route("/", get(index))
        .nest_service("/taskify_web_wasm", ServeDir::new("./taskify_web_wasm"));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    tracing::info!(%addr, "taskify_web starting");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index() -> Html<&'static str> {
    Html(
        r#"
            <!doctype html>
            <html>
                <head>

                </head>
                <body>
                    <canvas id="the_canvas_id" width="400" height="600"></canvas>
                    <div class="centered" id="center_text">
                        <p style="font-size:16px">
                            Loading…
                        </p>
                        <div class="lds-dual-ring"></div>
                    </div>
                    <script type="module">
                    import init, { WebHandle } from "./taskify_web_wasm/taskify_extension.js";

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

                        const wasmUrl = new URL("./taskify_web_wasm/taskify_extension_bg.wasm", import.meta.url);
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

                    </script>

                </body>
            </html>
            "#,
    )
}
