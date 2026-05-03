## --release or --dev - exclude/include debug info
## --no-typescript - disable .d.ts files output
## --out-dir - where to write the compiled files
## --out-name - force output file names
## --target - always use "web"!
## See https://rustwasm.github.io/wasm-pack/book/commands/build.html
echo Building wasm modules...
wasm-pack build taskify_extension --dev --no-typescript --out-dir "../extension/js/wasm" --out-name "taskify_extension" --target web
wasm-pack build taskify_extension_background --dev --no-typescript --out-dir "../extension/js/wasm" --out-name "taskify_extension_background" --target web
wasm-pack build taskify_core --dev --no-typescript --out-dir "../taskify_web/taskify_web_wasm" --out-name "taskify_core" --target web
rm -f extension/js/wasm/.gitignore
rm -f extension/js/wasm/package.json
rm -f taskify_web/taskify_web_wasm/.gitignore
rm -f taskify_web/taskify_web_wasm/package.json
rm -f chrome.zip && \
(cd extension && zip -rq ../chrome.zip . -x manifest_ff.json -x manifest.json) && \
printf "@ manifest_cr.json\n@=manifest.json\n" | zipnote -w chrome.zip && \
echo Chrome package: chrome.zip
rm -f firefox.zip && \
(cd extension && zip -rq ../firefox.zip . -x manifest_cr.json -x manifest.json) && \
printf "@ manifest_ff.json\n@=manifest.json\n" | zipnote -w firefox.zip && \
echo Firefox package: firefox.zip
