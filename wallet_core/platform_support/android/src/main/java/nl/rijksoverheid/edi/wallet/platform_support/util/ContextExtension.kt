package nl.rijksoverheid.edi.wallet.platform_support.util

import android.app.KeyguardManager
import android.content.Context

fun Context.isDeviceLocked(): Boolean {
    val myKM = this.getSystemService(Context.KEYGUARD_SERVICE) as? KeyguardManager
    return myKM?.isKeyguardLocked == true
}