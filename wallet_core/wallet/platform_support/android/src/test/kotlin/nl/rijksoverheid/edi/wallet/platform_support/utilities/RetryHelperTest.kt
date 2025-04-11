package nl.rijksoverheid.edi.wallet.platform_support.utilities

import android.util.Log
import io.kotest.assertions.throwables.shouldThrow
import io.kotest.inspectors.forAll
import io.kotest.matchers.collections.shouldHaveSize
import io.kotest.matchers.shouldBe
import io.kotest.matchers.string.shouldContain
import io.kotest.matchers.types.shouldBeSameInstanceAs
import io.mockk.every
import io.mockk.mockkStatic
import kotlinx.coroutines.test.runTest
import org.junit.Before
import org.junit.Test

class RetryHelperTest {

    private val debugs = mutableListOf<String>()
    private val errors = mutableListOf<String>()
    private val warnings = mutableListOf<String>()

    @Before
    fun init() {
        mockkStatic(Log::class)
        every { Log.d(any(), any(String::class)) }.answers {
            debugs += it.invocation.args[1] as String
            0
        }
        every { Log.e(any(), any(String::class)) }.answers {
            errors += it.invocation.args[1] as String
            0
        }
        every { Log.w(any(), any(String::class)) }.answers {
            warnings += it.invocation.args[1] as String
            0
        }
    }

    @Test
    fun `should only run once when no exceptions`() = runTest {
        var count = 0
        val result = Any()
        retryable(3) {
            count++
            result
        } shouldBeSameInstanceAs result
        count shouldBe 1
        debugs shouldHaveSize 1
        warnings shouldHaveSize 0
        errors shouldHaveSize 0
    }

    @Test
    fun `should return result even after one failure`() = runTest {
        var count = 0
        val result = Any()
        retryable(3) {
            if (count++ < 1) {
                throw RuntimeException("Temporarily")
            }
            result
        } shouldBeSameInstanceAs result
        count shouldBe 2
    }

    @Test
    fun `should only run max times when when block throws exception`() = runTest {
        var count = 0
        shouldThrow<IllegalStateException> {
            retryable(3) {
                count++
                throw IllegalStateException("ILLEGAL")
            }
        }
        count shouldBe 3
        debugs shouldHaveSize 3
        warnings.shouldHaveSize(2).forAll {
            it shouldContain "IllegalStateException.+ exception message: \"ILLEGAL\".+ remaining times: [12]".toRegex()
        }
        errors.shouldHaveSize(1).forAll {
            it shouldContain "IllegalStateException.+ exception message: \"ILLEGAL\"".toRegex()
        }
    }

}
