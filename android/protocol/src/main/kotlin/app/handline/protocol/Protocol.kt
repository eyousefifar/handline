// SPDX-License-Identifier: Apache-2.0

package app.handline.protocol

import org.bouncycastle.crypto.agreement.ECDHBasicAgreement
import org.bouncycastle.crypto.digests.SHA256Digest
import org.bouncycastle.crypto.engines.AESEngine
import org.bouncycastle.crypto.generators.HKDFBytesGenerator
import org.bouncycastle.crypto.modes.GCMBlockCipher
import org.bouncycastle.crypto.params.AEADParameters
import org.bouncycastle.asn1.x9.ECNamedCurveTable
import org.bouncycastle.crypto.params.ECDomainParameters
import org.bouncycastle.crypto.params.ECPrivateKeyParameters
import org.bouncycastle.crypto.params.ECPublicKeyParameters
import org.bouncycastle.crypto.params.HKDFParameters
import org.bouncycastle.crypto.params.KeyParameter
import org.bouncycastle.jce.provider.BouncyCastleProvider
import java.math.BigInteger
import java.security.Security
import java.util.Base64

object Wire {
    const val PREFIX = "handline:"
    const val WINDOW = 32
    val INFO = "handline v1".toByteArray()
    private val domain: ECDomainParameters = ECNamedCurveTable.getByName("P-256").let { x9 ->
        ECDomainParameters(x9.curve, x9.g, x9.n, x9.h, x9.seed)
    }

    init {
        if (Security.getProvider(BouncyCastleProvider.PROVIDER_NAME) == null) {
            Security.addProvider(BouncyCastleProvider())
        }
    }

    fun decodeQr(text: String): ParsedQr {
        require(text.startsWith(PREFIX)) { "bad_magic" }
        val raw = Base64.getUrlDecoder().decode(text.removePrefix(PREFIX))
        val r = Reader(raw)
        require(r.take(4).decodeToString() == "HL01") { "bad_magic" }
        require(r.u8() == 1) { "bad_version" }
        val deviceId = r.take(16)
        val publicKey = r.take(33)
        decodePoint(publicKey)
        val expiry = r.u64()
        val token = r.take(16)
        val psm = r.u16()
        val nameLen = r.u8()
        require(nameLen <= 64) { "oversized" }
        val name = r.take(nameLen).decodeToString()
        require(r.finished) { "bad_length" }
        return ParsedQr(deviceId, publicKey, expiry, token, psm, name)
    }

    fun clientHello(ephScalar: ByteArray, staticScalar: ByteArray, nonceI: ByteArray): ByteArray {
        val payload = compressedPublic(ephScalar) + compressedPublic(staticScalar) + nonceI
        return record(1, payload)
    }

    fun serverHello(ephScalar: ByteArray, nonceB: ByteArray): ByteArray {
        return record(2, compressedPublic(ephScalar) + nonceB)
    }

    fun trafficKeys(
        iEph: ByteArray, iStatic: ByteArray, bEph: ByteArray, bStatic: ByteArray,
        nonceI: ByteArray, nonceB: ByteArray, token: ByteArray
    ): Pair<ByteArray, ByteArray> {
        val dh1 = shared(iEph, compressedPublic(bEph))
        val dh2 = shared(iEph, compressedPublic(bStatic))
        val dh3 = shared(iStatic, compressedPublic(bEph))
        val dh4 = shared(iStatic, compressedPublic(bStatic))
        val ikm = dh1 + dh2 + dh3 + dh4
        val salt = nonceI + nonceB + token
        val hk = HKDFBytesGenerator(SHA256Digest())
        hk.init(HKDFParameters(ikm, salt, INFO))
        val out = ByteArray(64)
        hk.generateBytes(out, 0, 64)
        return out.copyOfRange(0, 32) to out.copyOfRange(32, 64)
    }

    fun finished(key: ByteArray, direction: Int, clientPayload: ByteArray, serverPayload: ByteArray, token: ByteArray, aad: ByteArray): ByteArray {
        val hash = sha256(clientPayload + serverPayload + token)
        return seal(key, direction, 0, aad, hash)
    }

    fun seal(key: ByteArray, direction: Int, counter: Int, aad: ByteArray, plain: ByteArray): ByteArray {
        val cipher = GCMBlockCipher.newInstance(AESEngine.newInstance())
        cipher.init(true, AEADParameters(KeyParameter(key), 128, nonce(direction, counter), aad))
        val out = ByteArray(cipher.getOutputSize(plain.size))
        var n = cipher.processBytes(plain, 0, plain.size, out, 0)
        n += cipher.doFinal(out, n)
        return out.copyOf(n)
    }

    fun open(key: ByteArray, direction: Int, counter: Int, aad: ByteArray, sealed: ByteArray): ByteArray {
        val cipher = GCMBlockCipher.newInstance(AESEngine.newInstance())
        cipher.init(false, AEADParameters(KeyParameter(key), 128, nonce(direction, counter), aad))
        val out = ByteArray(cipher.getOutputSize(sealed.size))
        var n = cipher.processBytes(sealed, 0, sealed.size, out, 0)
        n += cipher.doFinal(out, n)
        return out.copyOf(n)
    }

    fun record(kind: Int, payload: ByteArray): ByteArray {
        return byteArrayOf(kind.toByte()) + u16(payload.size) + payload
    }

    fun frameAad(type: Int, flags: Int, counter: Int): ByteArray {
        return byteArrayOf(1, type.toByte(), flags.toByte()) + u32(counter)
    }

    private fun sha256(data: ByteArray): ByteArray {
        val d = SHA256Digest()
        d.update(data, 0, data.size)
        val out = ByteArray(32)
        d.doFinal(out, 0)
        return out
    }

    private fun nonce(direction: Int, counter: Int): ByteArray {
        val n = ByteArray(12)
        n[0] = direction.toByte()
        u32(counter).copyInto(n, 8)
        return n
    }

    private fun compressedPublic(scalar: ByteArray): ByteArray = publicFrom(scalar).q.getEncoded(true)

    private fun shared(scalar: ByteArray, peerCompressed: ByteArray): ByteArray {
        val priv = privateFrom(scalar)
        val point = priv.parameters.curve.decodePoint(peerCompressed)
        val agree = ECDHBasicAgreement()
        agree.init(priv)
        return agree.calculateAgreement(ECPublicKeyParameters(point, priv.parameters)).fixed32()
    }

    private fun publicFrom(scalar: ByteArray): ECPublicKeyParameters {
        val priv = privateFrom(scalar)
        val q = priv.parameters.g.multiply(priv.d).normalize()
        return ECPublicKeyParameters(q, priv.parameters)
    }

    private fun privateFrom(scalar: ByteArray): ECPrivateKeyParameters {
        return ECPrivateKeyParameters(BigInteger(1, scalar), domain)
    }

    private fun decodePoint(bytes: ByteArray) {
        domain.curve.decodePoint(bytes)
    }

    private fun u16(v: Int) = byteArrayOf((v shr 8).toByte(), v.toByte())
    private fun u32(v: Int) = byteArrayOf((v shr 24).toByte(), (v shr 16).toByte(), (v shr 8).toByte(), v.toByte())
}

data class ParsedQr(
    val deviceId: ByteArray,
    val publicKey: ByteArray,
    val expiry: Long,
    val token: ByteArray,
    val psm: Int,
    val name: String
)

class ReplayWindow {
    var next: Int = 1
    private var bitmap: Long = 0

    fun accept(counter: Int) {
        if (counter == 0 || counter < next) error("replay")
        val off = counter - next
        if (off >= Wire.WINDOW) error("counter_ahead")
        val bit = 1L shl off
        if (bitmap and bit != 0L) error("replay")
        bitmap = bitmap or bit
        while (bitmap and 1L == 1L) {
            bitmap = bitmap ushr 1
            next += 1
        }
    }
}

private class Reader(private val data: ByteArray) {
    private var i = 0
    val finished: Boolean get() = i == data.size
    fun take(n: Int): ByteArray {
        require(i + n <= data.size) { "truncated" }
        val out = data.copyOfRange(i, i + n)
        i += n
        return out
    }
    fun u8(): Int = take(1)[0].toInt() and 0xff
    fun u16(): Int = (u8() shl 8) or u8()
    fun u64(): Long {
        var v = 0L
        repeat(8) { v = (v shl 8) or u8().toLong() }
        return v
    }
}

private fun BigInteger.fixed32(): ByteArray {
    val raw = toByteArray()
    val body = if (raw.isNotEmpty() && raw[0] == 0.toByte()) raw.copyOfRange(1, raw.size) else raw
    val out = ByteArray(32)
    body.copyInto(out, 32 - body.size)
    return out
}
