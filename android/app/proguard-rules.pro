# Keep JNI method names for any classes that still expose native methods.
-keepclasseswithmembernames class * {
    native <methods>;
}

# Uncomment this to preserve the line number information for
# debugging stack traces.
-keepattributes SourceFile,LineNumberTable

# JNA's native bridge (`libjnidispatch.so`) looks up many JNA classes, fields,
# and methods by their original names. The stable practice is to keep the JNA
# namespace intact rather than chasing individual members across versions.
-keep class com.sun.jna.** { *; }
-keep interface com.sun.jna.** { *; }

# JNA contains desktop-only AWT helpers which are never used on Android.
# Suppress warnings for those optional references when the package is kept.
-dontwarn java.awt.**

# Keep the generated UniFFI bindings package intact as the JNA-facing ABI layer.
-keep class com.fuwaki.synap.bindings.uniffi.synap_coreffi.** { *; }
-keep interface com.fuwaki.synap.bindings.uniffi.synap_coreffi.** { *; }

# Keep Kotlin coroutines
-keepnames class kotlinx.coroutines.internal.MainDispatcherFactory {}
-keepnames class kotlinx.coroutines.CoroutineDispatcher {}
-keepnames class kotlinx.coroutines.android.AndroidDispatcherFactory {}
-keepclassmembernames class kotlinx.** {
    volatile <fields>;
}

# Keep serialization
-keepattributes *Annotation*
-keepclassmembers class * {
    @androidx.compose.runtime.Composable public *();
}
