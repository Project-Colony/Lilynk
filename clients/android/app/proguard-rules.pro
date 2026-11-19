# Punktfunk ProGuard Rules

# Keep the Native Bridge and its methods for JNI
-keep class org.projectcolony.lilynk.kit.NativeBridge { *; }
-keepclasseswithmembernames class * {
    native <methods>;
}

# Keep the models that might be serialized or accessed via JNI
-keep class org.projectcolony.lilynk.models.** { *; }
-keep class org.projectcolony.lilynk.kit.discovery.** { *; }
-keep class org.projectcolony.lilynk.kit.security.** { *; }

# Compose rules are usually handled by the plugin, but we can add more if needed
-keepclassmembers class **.R$* {
    public static <fields>;
}
