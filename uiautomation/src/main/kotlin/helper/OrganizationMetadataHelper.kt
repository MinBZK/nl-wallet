package helper

import helper.FileUtils.getProjectFile
import java.io.File

class OrganizationMetadataHelper {
    companion object {
        val ACCESS_CERTIFICATES_SH = getProjectFile("scripts/devenv/access_certificates.sh")
    }

    private val organizationNames: Map<Organization, String> by lazy {
        val nameRegex = """\[\(([^,]+),name\)\]='([^']+)'""".toRegex()

        var names = mutableMapOf<Organization,String>()
        File(ACCESS_CERTIFICATES_SH).forEachLine { line ->
            val match = nameRegex.find(line)
            if (match != null) {
                val organization = try {
                     Organization.valueOf(match.groupValues[1].uppercase())
                } catch (_: IllegalArgumentException) {
                    return@forEachLine
                }
                names[organization] = match.groupValues[2]
            }
        }
        names
    }

    fun getDisplayNameOfOrganization(rp: Organization): String {
        return organizationNames[rp] ?: throw IllegalStateException("No name known for RP $rp")
    }

    enum class Organization {
        PID,
        MIJN_AMSTERDAM,
        ONLINE_MARKETPLACE,
        XYZ_BANK,
        MONKEY_BIKE,
        JOB_FINDER,
        UNIVERSITY,
        INSURANCE,
        HOUSING,
        LOYALTY,
        MUSEUM_MAANDKAART
    }
}
