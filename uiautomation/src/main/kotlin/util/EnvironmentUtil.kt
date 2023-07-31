package util

class EnvironmentUtil {
    companion object {
        fun getVar(name: String) = System.getenv(name) ?: ""
    }
}
