package helper

// Same pattern as envsubst uses:
// https://git.savannah.gnu.org/gitweb/?p=gettext.git;a=blob;f=gettext-runtime/src/envsubst.c;h=0f90d64c00ca7835d9c0f544235a80f0973e13c3;hb=957ecb5ea6577361dfd0655b4862d8797df5eb2d#l237
private val variablePattern = Regex("""\$(\{[A-Za-z_][A-Za-z0-9_]*}|[A-Za-z_][A-Za-z0-9_]*)""")

fun envsubst(text: String, lookup: (String) -> String = System::getenv) = variablePattern.replace(text) { match ->
    val name = match.groupValues[1].let { if (it[0] == '{') it.substring(1, it.length - 1) else it }
    lookup(name)
}
