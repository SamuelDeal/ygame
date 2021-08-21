
// Override this value if you want to connect to a specific server.
// Default behaviour is to connect to current address and port, 
// using TLS only if current page already use https
const server_override_value = null;

// Load the current subfolder value
function getCurrentSubfolder() {
    let scripts = document.getElementsByTagName("script");
    let current_script_url = scripts[scripts.length - 1].src; // Current script path during loading

    var match = current_script_url.match('^https?://[^/]*(/[^?#]*)/client.js(?:#.*|\\?.*|)$');
    if (!match) {
        return '';
    }
    return "" + match[1].replace(/^\/*|\/*$/g, '');
}
const subfolder = getCurrentSubfolder();


import init, { WebClient } from './wasm/ygame_client.js';

window.addEventListener('load', async () => {
    let wasm_path = ((subfolder == "") ? "" : "/" + subfolder) + '/wasm/ygame_client_bg.wasm';
    await init(wasm_path);
    window.webClient = new WebClient(server_override_value, subfolder);
    webClient.start();
});
