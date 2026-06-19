package com.pickando.demo;

import android.app.Activity;
import android.os.Bundle;
import android.webkit.WebView;
import android.webkit.WebSettings;
import android.webkit.WebViewClient;
import android.webkit.WebChromeClient;
import android.view.KeyEvent;
import android.view.WindowManager;
import android.view.View;
import android.graphics.Color;

/**
 * Nitheky Android wrapper.
 *
 * Loads the mobile-optimized route `/m/` from the deployed WASM app.
 * The mobile route is a distinct URL space (`/m/*`) so the WASM bundle
 * renders Android-optimized layouts (bottom-nav, safe-area insets,
 * touch-first components) instead of the desktop platform UI.
 *
 * Strict separation:
 *  - `/`        → Landing (public marketing)           ← not used by Android
 *  - `/app/*`   → Platform (desktop web app)           ← not used by Android
 *  - `/m/*`     → Mobile (Android-optimized)           ← loaded here
 */
public class MainActivity extends Activity {
    private WebView webView;

    // The deployed Nitheky demo URL — mobile route.
    // Update this when the deployment changes.
    private static final String APP_URL = "https://pickando-demo-production.up.railway.app/m/";

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        // Keep screen on while the app is foregrounded
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);

        // Edge-to-edge rendering with dark status bar matching brand (#0A0A0A ink)
        getWindow().setStatusBarColor(Color.parseColor("#0A0A0A"));
        getWindow().setNavigationBarColor(Color.parseColor("#0A0A0A"));
        getWindow().getDecorView().setSystemUiVisibility(
            View.SYSTEM_UI_FLAG_LAYOUT_STABLE | View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
        );

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

        // WebViewClient: keep navigation inside the WebView (don't open external browser)
        webView.setWebViewClient(new WebViewClient() {
            @Override
            public boolean shouldOverrideUrlLoading(WebView view, String url) {
                view.loadUrl(url);
                return true;
            }
        });

        // WebChromeClient: required for console.log() passthrough and alert()
        webView.setWebChromeClient(new WebChromeClient());

        // Load the mobile route — never the marketing landing
        webView.loadUrl(APP_URL);
    }

    /**
     * Hardware back button: navigate WebView history if possible,
     * otherwise exit the activity. This is critical for the mobile
     * UX — without it, pressing Back exits the app immediately
     * even when there's a meaningful history stack (e.g. /m/ → /m/passenger).
     */
    @Override
    public boolean onKeyDown(int keyCode, KeyEvent event) {
        if (keyCode == KeyEvent.KEYCODE_BACK && webView.canGoBack()) {
            webView.goBack();
            return true;
        }
        return super.onKeyDown(keyCode, event);
    }

    @Override
    protected void onPause() {
        super.onPause();
        // Pause any active JS timers / animations to save battery
        webView.onPause();
    }

    @Override
    protected void onResume() {
        super.onResume();
        webView.onResume();
    }
}
