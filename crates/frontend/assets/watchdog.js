// =====================================================================
// Nitheky — Loading-screen watchdog (external file, CSP-friendly).
//
// Why external? The page CSP is `script-src 'self' 'wasm-unsafe-eval'`,
// which BLOCKS inline scripts. Moving this to /assets/watchdog.js lets
// it run under 'self' without weakening CSP with 'unsafe-inline'.
//
// What it does: polls every 100ms for Dioxus to mount into #main, then
// removes the loading screen. Also surfaces WASM/CSP/Dioxus errors that
// would otherwise leave the user staring at a perpetual spinner.
// =====================================================================
(function () {
    var status = document.getElementById('loading-status');
    var errorBox = document.getElementById('loading-error');
    var errorMsg = document.getElementById('loading-error-msg');
    var loadingScreen = document.getElementById('loading-screen');
    var main = document.getElementById('main');
    var hidden = false;
    var startTime = Date.now();
    var wasmLoadedAt = null;

    var statusMessages = [
        'Cargando Nitheky…',
        'Inicializando motor de matching…',
        'Conectando con backend Rust…',
        'Casi listo…'
    ];
    var msgIdx = 0;
    var msgInterval = setInterval(function () {
        if (hidden) { clearInterval(msgInterval); return; }
        msgIdx = (msgIdx + 1) % statusMessages.length;
        if (status) status.textContent = statusMessages[msgIdx];
    }, 1200);

    function showFatalError(reason, detail) {
        if (status) status.style.display = 'none';
        if (errorBox) errorBox.style.display = 'block';
        if (errorMsg) {
            errorMsg.innerHTML = reason +
                '<code>' + detail + '</code>' +
                '<p style="margin-top:10px;color:#6E6E6E;font-size:0.85rem;">' +
                'Posibles causas:<br>' +
                '- El archivo WASM no se generó o no se sirvió correctamente.<br>' +
                '- Hay un error de red o el backend no está respondiendo.<br>' +
                '- El navegador no soporta WebAssembly.<br>' +
                'Revisa la consola del navegador (F12) para más detalles.</p>';
        }
        console.error('[Nitheky] Fatal load error:', reason, detail);
    }

    function hideLoadingScreen() {
        if (hidden || !loadingScreen || !loadingScreen.parentNode) return;
        hidden = true;
        clearInterval(msgInterval);
        loadingScreen.style.transition = 'opacity 0.35s ease';
        loadingScreen.style.opacity = '0';
        setTimeout(function () {
            if (loadingScreen.parentNode) loadingScreen.parentNode.removeChild(loadingScreen);
        }, 400);
        console.log('[Nitheky] App mounted in ' + (Date.now() - startTime) + 'ms');
    }

    if (typeof WebAssembly === 'undefined') {
        showFatalError(
            'Este navegador no soporta WebAssembly.',
            'WebAssembly object is undefined.'
        );
        return;
    }

    window.addEventListener('error', function (event) {
        if (event.message && (
            event.message.indexOf('CompileError') >= 0 ||
            event.message.indexOf('LinkError') >= 0 ||
            event.message.indexOf('wasm') >= 0 ||
            event.message.indexOf('WASM') >= 0 ||
            event.message.indexOf('WebAssembly') >= 0 ||
            event.message.indexOf('Content-Security-Policy') >= 0 ||
            event.message.indexOf('Dioxus') >= 0 ||
            event.message.indexOf('dioxus') >= 0
        )) {
            showFatalError(
                'Error durante la inicialización de la app.',
                String(event.message || 'Unknown error') + '\n' +
                String(event.error && event.error.stack ? event.error.stack : '')
            );
        }
    });

    window.addEventListener('unhandledrejection', function (event) {
        var reasonStr = '';
        try { reasonStr = String(event.reason && event.reason.message ? event.reason.message : event.reason); }
        catch (e) { reasonStr = 'unknown'; }
        // Match case-insensitively against common WASM/CSP/Dioxus error markers.
        // Previous version missed real errors because it only matched lowercase
        // 'wasm' / uppercase 'WASM' but actual browser errors say 'WebAssembly'.
        var lowerReason = reasonStr.toLowerCase();
        if (lowerReason.indexOf('wasm') >= 0 ||
            lowerReason.indexOf('webassembly') >= 0 ||
            lowerReason.indexOf('dioxus') >= 0 ||
            lowerReason.indexOf('content-security-policy') >= 0 ||
            lowerReason.indexOf('csp') >= 0 ||
            lowerReason.indexOf('fetch') >= 0 ||
            lowerReason.indexOf('compileerror') >= 0) {
            showFatalError('Error al cargar el WASM o inicializar Dioxus.', reasonStr);
        }
    });

    var pollInterval = setInterval(function () {
        if (hidden) {
            clearInterval(pollInterval);
            return;
        }
        if (main && main.children && main.children.length > 0) {
            clearInterval(pollInterval);
            hideLoadingScreen();
            return;
        }
        if (window.__dx_mainWasm && !wasmLoadedAt) {
            wasmLoadedAt = Date.now();
        }
        if (wasmLoadedAt && (Date.now() - wasmLoadedAt) > 2000) {
            var dioxusNodes = document.querySelectorAll('[data-dioxus-id]');
            if (dioxusNodes.length > 0) {
                clearInterval(pollInterval);
                hideLoadingScreen();
                return;
            }
        }
    }, 100);

    setTimeout(function () {
        if (!hidden && loadingScreen && loadingScreen.parentNode) {
            clearInterval(pollInterval);
            clearInterval(msgInterval);
            var diag = 'Timeout after 30 seconds.\n' +
                'Elapsed: ' + (Date.now() - startTime) + 'ms\n' +
                '#main children: ' + (main ? main.children.length : 'no #main') + '\n' +
                'WASM loaded: ' + (window.__dx_mainWasm ? 'yes' : 'no') + '\n' +
                'Dioxus nodes in body: ' + document.querySelectorAll('[data-dioxus-id]').length + '\n' +
                'The WASM file may be too large, the connection too slow, or main.js failed to load.\n' +
                'Check the browser console (F12) for errors.';
            showFatalError('La aplicación tardó demasiado en iniciar.', diag);
        }
    }, 30000);
})();
