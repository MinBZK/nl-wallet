-keep class uniffi.platform_support.** { *; }

-keepnames class * extends androidx.startup.Initializer
-keep class * extends androidx.startup.Initializer {
    # Keep the public no-argument constructor while allowing other methods to be optimized.
    <init>();
}

-keep class com.sun.jna.** { *; }

-dontwarn java.awt.*
