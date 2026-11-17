pluginManagement {
    repositories {
        if (System.getenv("CI") == "true" && System.getenv("CI_JOB_TAGS") == "[]") {
            maven {
                url = uri("${System.getenv("NEXUS_URL")}/repository/maven-central/")
            }
        } else {
            gradlePluginPortal()
        }
    }
}

rootProject.name = "UiAutomation"
