package com.pickando.demo;

import android.app.Activity;
import android.content.Context;
import android.net.ConnectivityManager;
import android.net.NetworkInfo;
import android.os.Bundle;
import android.webkit.WebView;
import android.webkit.WebResourceRequest;
import android.webkit.WebResourceError;
import android.webkit.WebResourceResponse;
import android.webkit.WebSettings;
import android.webkit.WebViewClient;
import android.webkit.WebChromeClient;
import android.view.KeyEvent;
import android.view.WindowManager;
import android.view.View;
import android.graphics.Color;
import android.util.Log;
import java.io.InputStream;
import java.io.IOException;

/**
 * Nitheky Android wrapper — production-hardened.
 *
 * Loads the mobile-optimized route `/m/` from the deployed WASM app.
 *
 * Hardening:
 *  - Offline detection with graceful fallback page (assets/offline.html)
 *  - Error handler for WebView load failures (network/DNS/HTTP errors)
 *  - WebView SSL error handling: refuse invalid certs (never proceed)
 *  - Back button: navigate history, then exit
 *  - Memory-efficient: clear cache and history on destroy
 *  - Crash-safe: try/catch around all WebView setup
 */
public class MainActivity extends Activity {
    private static final String TAG = "Nitheky";
    private WebView webView;

    /** The deployed Nitheky demo URL — mobile route. */
    private static final String APP_URL = "https://pickando-demo-production.up.railway.app/m/";

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        try {
            setupWindow();
            setupWebView();
        } catch (Exception e) {
            Log.e(TAG, "Failed to initialize WebView", e);
            // Last-resort: show offline page so the user sees something
            showOfflinePage();
        }
    }

    private void setupWindow() {
        // Keep screen on while the app is foregrounded
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
        // Edge-to-edge rendering with dark status bar matching brand (#0A0A0A ink)
        getWindow().setStatusBarColor(Color.parseColor("#0A0A0A"));
        getWindow().setNavigationBarColor(Color.parseColor("#0A0A0A"));
    }

    private void setupWebView() {
        webView = new WebView(this);
        setContentView(webView);

        WebSettings settings = webView.getSettings();
        settings.setJavaScriptEnabled(true);
        settings.setDomStorageEnabled(true);
        settings.setDatabaseEnabled(true);
        settings.setAllowFileAccess(true);
        settings.setAllowContentAccess(true);
        settings.setLoadWithOverviewMode(true);
        settings.setUseWideViewPort(true);
        settings.setSupportZoom(false);
        settings.setBuiltInZoomControls(false);
        settings.setMediaPlaybackRequiresUserGesture(false);
        settings.setCacheMode(WebSettings.LOAD_DEFAULT);
        // Enable viewport meta tag handling for responsive layout
        settings.setUserAgentString(settings.getUserAgentString() + " Nitheky/Android");
        // Mixed content: never allow HTTP subresources on HTTPS pages
        settings.setMixedContentMode(WebSettings.MIXED_CONTENT_NEVER_ALLOW);

        // WebViewClient: keep navigation inside the WebView + handle errors
        webView.setWebViewClient(new WebViewClient() {
            @Override
            public boolean shouldOverrideUrlLoading(WebView view, WebResourceRequest request) {
                // Only allow http(s) URLs to load inside the WebView
                String url = request.getUrl().toString();
                if (url.startsWith("http://") || url.startsWith("https://")) {
                    view.loadUrl(url);
                    return true;
                }
                // Defer to system for non-http schemes (mailto:, tel:, intent:)
                return false;
            }

            @Override
            public void onReceivedError(WebView view, WebResourceRequest request, WebResourceError error) {
                // Only show offline page if the MAIN frame failed (not subresources)
                if (request.isForMainFrame()) {
                    Log.w(TAG, "Main frame load error: " + error.getDescription()
                            + " (code=" + error.getErrorCode() + ")");
                    showOfflinePage();
                }
                super.onReceivedError(view, request, error);
            }

            @Override
            public void onReceivedHttpError(WebView view, WebResourceRequest request, WebResourceResponse errorResponse) {
                if (request.isForMainFrame()) {
                    Log.w(TAG, "Main frame HTTP error: " + errorResponse.getStatusCode());
                    if (errorResponse.getStatusCode() >= 500) {
                        showOfflinePage();
                    }
                }
                super.onReceivedHttpError(view, request, errorResponse);
            }
        });

        // WebChromeClient: required for console.log() passthrough and alert()
        webView.setWebChromeClient(new WebChromeClient());

        // Initial load — check connectivity first
        if (isOnline()) {
            webView.loadUrl(APP_URL);
        } else {
            showOfflinePage();
        }
    }

    /** Returns true if the device has any active network connection. */
    private boolean isOnline() {
        try {
            ConnectivityManager cm = (ConnectivityManager) getSystemService(Context.CONNECTIVITY_SERVICE);
            if (cm == null) return false;
            NetworkInfo net = cm.getActiveNetworkInfo();
            return net != null && net.isConnected();
        } catch (Exception e) {
            Log.w(TAG, "Connectivity check failed", e);
            return false;
        }
    }

    /** Loads the bundled offline page (assets/offline.html). */
    private void showOfflinePage() {
        if (webView == null) {
            webView = new WebView(this);
            setContentView(webView);
        }
        try {
            InputStream is = getAssets().open("offline.html");
            byte[] buffer = new byte[is.available()];
            is.read(buffer);
            is.close();
            String html = new String(buffer, "UTF-8");
            webView.loadDataWithBaseURL("file:///android_asset/", html, "text/html", "UTF-8", null);
        } catch (IOException e) {
            Log.e(TAG, "Failed to load offline page", e);
            // Ultimate fallback: inline HTML
            String fallback = "<html><body style='background:#0A0A0A;color:#C9A961;"
                    + "font-family:sans-serif;text-align:center;padding:48px'>"
                    + "<h1>Nitheky</h1>"
                    + "<p>No hay conexión a internet.</p>"
                    + "<p>Revisa tu red y vuelve a intentarlo.</p>"
                    + "</body></html>";
            webView.loadData(fallback, "text/html", "UTF-8");
        }
    }

    /**
     * Hardware back button: navigate WebView history if possible,
     * otherwise exit the activity.
     */
    @Override
    public boolean onKeyDown(int keyCode, KeyEvent event) {
        if (keyCode == KeyEvent.KEYCODE_BACK && webView != null && webView.canGoBack()) {
            webView.goBack();
            return true;
        }
        return super.onKeyDown(keyCode, event);
    }

    @Override
    protected void onPause() {
        super.onPause();
        if (webView != null) webView.onPause();
    }

    @Override
    protected void onResume() {
        super.onResume();
        if (webView != null) webView.onResume();
    }

    @Override
    protected void onDestroy() {
        // Clean up WebView properly to prevent memory leaks
        if (webView != null) {
            try {
                webView.clearHistory();
                webView.clearCache(true);
                webView.clearFormData();
                webView.loadUrl("about:blank");
                webView.onPause();
                webView.removeAllViews();
                ((android.view.ViewGroup) webView.getParent()).removeView(webView);
                webView.destroy();
                webView = null;
            } catch (Exception e) {
                Log.w(TAG, "WebView cleanup error", e);
            }
        }
        super.onDestroy();
    }
}
