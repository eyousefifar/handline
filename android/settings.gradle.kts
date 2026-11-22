pluginManagement {
    repositories {
        gradlePluginPortal()
        mavenCentral()
    }
}

dependencyResolutionManagement {
    repositories {
        mavenCentral()
    }
}

rootProject.name = "handline"
include(":protocol", ":logic")

val sdk = System.getenv("ANDROID_HOME")?.let { file(it) }
    ?: file("${System.getProperty("user.home")}/Library/Android/sdk")
if (sdk.resolve("platforms").isDirectory) {
    include(":app")
}
