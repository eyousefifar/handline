// SPDX-License-Identifier: Apache-2.0

package app.handline.logic

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class LogicTest {
    @Test
    fun callsSimsCodesProbeAndInstall() {
        val book = CallBook()
        assertTrue(book.invite(7, incoming = true, sim = 0, number = "+1"))
        assertTrue(book.advance(7, 1))
        assertTrue(book.offHook)
        assertTrue(book.dtmf(7, '5'.code))
        assertEquals("5", book.calls[7]?.digits)
        assertTrue(book.advance(7, 3))
        assertFalse(book.offHook)
        assertEquals(0, book.calls[7]?.status)

        val missed = CallBook()
        missed.invite(4, incoming = true, sim = 0, number = "+1")
        missed.advance(4, 3)
        assertEquals(1, missed.calls[4]?.status)

        val sims = CallBook()
        sims.invite(1, incoming = false, sim = 0, number = "+200")
        sims.invite(2, incoming = false, sim = 0xFF, number = "+200")
        sims.invite(3, incoming = false, sim = 1, number = "+200")
        sims.invite(4, incoming = false, sim = 0xFF, number = "+200")
        assertEquals(1, sims.calls[4]?.sim)

        assertTrue(isOneTimeCode("123456"))
        assertTrue(isOneTimeCode("Your code is 482193"))
        assertFalse(isOneTimeCode("Lunch at 1234 Main"))

        assertEquals(ProbeFinding.TWO_WAY, probeAudio { it == AudioSource.VOICE_CALL })
        assertEquals(ProbeFinding.UPLINK_ONLY, probeAudio { it == AudioSource.MIC })
        assertEquals(ProbeFinding.NONE, probeAudio { false })

        val install = HelperInstall()
        install.submit("12")
        assertEquals(InstallPhase.IDLE, install.phase)
        install.submit("123456")
        install.installSucceeded()
        assertEquals(InstallPhase.PAIRING, install.phase)
        install.pairSucceeded()
        install.installSucceeded()
        install.apply(ProbeFinding.UPLINK_ONLY)
        assertEquals(InstallPhase.FAILED, install.phase)
        install.submit("654321")
        install.pairSucceeded()
        install.installSucceeded()
        install.apply(ProbeFinding.TWO_WAY)
        assertEquals(InstallPhase.READY, install.phase)

        assertEquals("گوشی دیگر در دسترس نیست", copy("out_of_range", persian = true))
        assertTrue(copy("linked", persian = false).isNotEmpty())
    }
}
