# Nitheky — ProGuard / R8 rules
#
# Release builds use minifyEnabled=false for now (safer with WebView).
# These rules are kept for when we enable R8 in the future.

# Keep the WebView wrapper activity and all its members
-keep class com.pickando.demo.** { *; }

# Keep WebView-related classes (used via reflection)
-keep class android.webkit.** { *; }
-keep class android.webkit.WebView { *; }
-keep class android.webkit.WebViewClient { *; }
-keep class android.webkit.WebChromeClient { *; }
-keepclassmembers class * extends android.webkit.WebViewClient {
    public *;
}
-keepclassmembers class * extends android.webkit.WebChromeClient {
    public *;
}

# Keep AndroidX appcompat (used)
-keep class androidx.appcompat.** { *; }

# Keep BuildConfig
-keep class com.pickando.demo.BuildConfig { *; }
