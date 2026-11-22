// SPDX-License-Identifier: Apache-2.0

package app.handline.logic

enum class CallPhase { RINGING, ACTIVE, HELD, ENDED }

data class HandCall(
    var phase: CallPhase,
    val incoming: Boolean,
    val sim: Int,
    var digits: String = "",
    var wentActive: Boolean = false,
    var status: Int? = null
)

class CallBook {
    val calls = linkedMapOf<Long, HandCall>()
    val threadSim = linkedMapOf<String, Int>()
    val offHook: Boolean
        get() = calls.values.any { it.phase == CallPhase.ACTIVE || it.phase == CallPhase.HELD }

    fun invite(id: Long, incoming: Boolean, sim: Int, number: String): Boolean {
        if (calls.containsKey(id)) return false
        val resolved = if (sim == 0xFF) threadSim[number] ?: 0 else sim
        threadSim[number] = resolved
        calls[id] = HandCall(CallPhase.RINGING, incoming, resolved)
        return true
    }

    fun advance(id: Long, state: Int): Boolean {
        val call = calls[id] ?: return false
        val ok = when (call.phase) {
            CallPhase.RINGING -> when (state) {
                1 -> { call.phase = CallPhase.ACTIVE; call.wentActive = true; true }
                3, 4, 5, 6 -> { call.phase = CallPhase.ENDED; true }
                else -> false
            }
            CallPhase.HELD -> when (state) {
                1 -> { call.phase = CallPhase.ACTIVE; true }
                3, 6 -> { call.phase = CallPhase.ENDED; true }
                else -> false
            }
            CallPhase.ACTIVE -> when (state) {
                2 -> { call.phase = CallPhase.HELD; true }
                3, 6 -> { call.phase = CallPhase.ENDED; true }
                else -> false
            }
            CallPhase.ENDED -> false
        }
        if (!ok) return false
        if (call.phase == CallPhase.ENDED) call.status = statusOf(call, state)
        return true
    }

    fun dtmf(id: Long, digit: Int): Boolean {
        val call = calls[id] ?: return false
        if (call.phase != CallPhase.ACTIVE && call.phase != CallPhase.HELD) return false
        if (digit !in "0123456789*#ABCD".map { it.code }) return false
        call.digits += digit.toChar()
        return true
    }

    private fun statusOf(call: HandCall, terminal: Int): Int = when {
        terminal == 4 -> 2
        terminal == 5 -> 3
        terminal == 6 -> 4
        terminal == 3 && call.wentActive && call.incoming -> 0
        terminal == 3 && call.wentActive && !call.incoming -> 5
        terminal == 3 && call.incoming -> 1
        else -> 4
    }
}

fun isOneTimeCode(body: String): Boolean {
    val trimmed = body.trim()
    if (trimmed.length in 4..8 && trimmed.all { it.isDigit() }) return true
    val lower = body.lowercase()
    val keyword = listOf("code", "otp", "verify", "verification").any { lower.contains(it) }
    return keyword && digitRun(body)
}

private fun digitRun(body: String): Boolean {
    var run = 0
    for (ch in body) {
        if (ch.isDigit()) {
            run += 1
        } else if (run in 4..8) {
            return true
        } else {
            run = 0
        }
    }
    return run in 4..8
}

enum class AudioSource { VOICE_CALL, VOICE_COMMUNICATION, MIC }
enum class ProbeFinding { TWO_WAY, UPLINK_ONLY, NONE }

fun probeAudio(open: (AudioSource) -> Boolean): ProbeFinding {
    if (open(AudioSource.VOICE_CALL)) return ProbeFinding.TWO_WAY
    if (open(AudioSource.VOICE_COMMUNICATION) || open(AudioSource.MIC)) return ProbeFinding.UPLINK_ONLY
    return ProbeFinding.NONE
}

enum class InstallPhase { IDLE, PAIRING, INSTALLING, PROBING, READY, FAILED }

class HelperInstall {
    var phase = InstallPhase.IDLE
        private set
    var failure: String? = null
        private set

    fun submit(code: String) {
        if (code.length != 6 || code.any { !it.isDigit() }) {
            failure = "code"
            return
        }
        if (phase != InstallPhase.IDLE && phase != InstallPhase.FAILED) return
        failure = null
        phase = InstallPhase.PAIRING
    }

    fun pairSucceeded() {
        if (phase == InstallPhase.PAIRING) phase = InstallPhase.INSTALLING
    }

    fun installSucceeded() {
        if (phase == InstallPhase.INSTALLING) phase = InstallPhase.PROBING
    }

    fun apply(probe: ProbeFinding) {
        if (phase != InstallPhase.PROBING) return
        if (probe == ProbeFinding.TWO_WAY) {
            phase = InstallPhase.READY
        } else {
            phase = InstallPhase.FAILED
            failure = "audio"
        }
    }
}

fun copy(key: String, persian: Boolean): String {
    val en = mapOf(
        "out_of_range" to "The other phone is out of reach",
        "linked" to "Linked",
        "audio_unavailable" to "Call audio is not available on this phone",
        "pair" to "Type the 6-digit code"
    )
    val fa = mapOf(
        "out_of_range" to "گوشی دیگر در دسترس نیست",
        "linked" to "وصل",
        "audio_unavailable" to "صدای تماس روی این گوشی در دسترس نیست",
        "pair" to "رمز شش‌رقمی را وارد کنید"
    )
    return (if (persian) fa else en)[key] ?: key
}
