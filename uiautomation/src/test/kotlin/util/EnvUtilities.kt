package util

object EnvUtilities {
    fun getEnvVar(name: String): String {
        return System.getenv(name) ?: ""
    }
}
