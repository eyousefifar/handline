// SPDX-License-Identifier: Apache-2.0

package app.handline.protocol

import org.json.JSONObject
import java.io.File
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFails
import kotlin.test.assertTrue

class FixtureTest {
    @Test
    fun referenceFixtureMatchesRust() {
        val file = File("../../handline-core/testdata/reference.json")
        assertTrue(file.isFile, file.absolutePath)
        val json = JSONObject(file.readText())
        fun hex(key: String): ByteArray = json.getString(key).hexToBytes()

        val qr = Wire.decodeQr(json.getString("qr"))
        assertEquals("Bag", qr.name)
        assertEquals(79, qr.psm)
        assertContentEquals(hex("token_hex"), qr.token)
        assertEquals(json.getLong("now") + 300, qr.expiry)

        val ch = Wire.clientHello(hex("hand_eph_scalar_hex"), hex("hand_scalar_hex"), hex("nonce_i_hex"))
        assertContentEquals(hex("client_hello_hex"), ch)
        val sh = Wire.serverHello(hex("bag_eph_scalar_hex"), hex("nonce_b_hex"))
        assertContentEquals(hex("server_hello_hex"), sh)

        val keys = Wire.trafficKeys(
            hex("hand_eph_scalar_hex"), hex("hand_scalar_hex"),
            hex("bag_eph_scalar_hex"), hex("bag_scalar_hex"),
            hex("nonce_i_hex"), hex("nonce_b_hex"), hex("token_hex")
        )
        assertContentEquals(hex("c2s_key_hex"), keys.first)
        assertContentEquals(hex("s2c_key_hex"), keys.second)

        val clientPayload = ch.copyOfRange(3, ch.size)
        val serverPayload = sh.copyOfRange(3, sh.size)
        val cf = Wire.finished(keys.first, 1, clientPayload, serverPayload, hex("token_hex"), "cfin".toByteArray())
        assertContentEquals(hex("client_finished_hex"), Wire.record(3, cf))
        val sf = Wire.finished(keys.second, 2, clientPayload, serverPayload, hex("token_hex"), "sfin".toByteArray())
        assertContentEquals(hex("server_finished_hex"), Wire.record(4, sf))

        val hello = hex("hello_hand_hex")
        val plain = openFrame(hello, keys.first, 1)
        assertEquals(1, plain[0].toInt())
        assertEquals(1, plain[1].toInt())

        val window = ReplayWindow()
        window.accept(counter(hello))
        assertFails { window.accept(counter(hello)) }

        val tampered = hex("tampered_ping_hex")
        assertFails {
            Wire.open(keys.first, 1, counter(tampered), Wire.frameAad(tampered[1].toInt() and 0xff, tampered[2].toInt() and 0xff, counter(tampered)), sealed(tampered))
        }
    }
}

private fun String.hexToBytes(): ByteArray {
    require(length % 2 == 0)
    return chunked(2).map { it.toInt(16).toByte() }.toByteArray()
}

private fun counter(frame: ByteArray): Int {
    return ((frame[3].toInt() and 0xff) shl 24) or
        ((frame[4].toInt() and 0xff) shl 16) or
        ((frame[5].toInt() and 0xff) shl 8) or
        (frame[6].toInt() and 0xff)
}

private fun sealed(frame: ByteArray): ByteArray {
    val ctLen = ((frame[7].toInt() and 0xff) shl 8) or (frame[8].toInt() and 0xff)
    return frame.copyOfRange(9, 9 + ctLen + 16)
}

private fun openFrame(frame: ByteArray, key: ByteArray, direction: Int): ByteArray {
    val type = frame[1].toInt() and 0xff
    val flags = frame[2].toInt() and 0xff
    val ctr = counter(frame)
    return Wire.open(key, direction, ctr, Wire.frameAad(type, flags, ctr), sealed(frame))
}
