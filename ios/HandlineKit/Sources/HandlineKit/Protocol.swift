// SPDX-License-Identifier: Apache-2.0

import CryptoKit
import Foundation

public enum HandlineError: Error, Equatable {
    case truncated
    case badMagic
    case badVersion
    case badLength
    case badPublicKey
    case crypto
    case replay
    case counterAhead
    case unexpectedFlags
    case unknownType
}

public enum Wire {
    public static let prefix = "handline:"
    public static let window: UInt32 = 32
    public static let info = Data("handline v1".utf8)
}

public struct ParsedQr: Equatable {
    public var deviceId: Data
    public var publicKey: Data
    public var expiry: UInt64
    public var token: Data
    public var psm: UInt16
    public var name: String
}

public enum HandlineCodec {
    public static func decodeQr(_ text: String) throws -> ParsedQr {
        guard let rest = text.stripPrefix(Wire.prefix) else { throw HandlineError.badMagic }
        guard let raw = Data(base64URLEncoded: rest) else { throw HandlineError.badMagic }
        var r = Reader(raw)
        let magic = try r.take(4)
        guard magic == Data("HL01".utf8) else { throw HandlineError.badMagic }
        guard try r.u8() == 1 else { throw HandlineError.badVersion }
        let deviceId = try r.take(16)
        let publicKey = try r.take(33)
        _ = try P256.KeyAgreement.PublicKey(compressedRepresentation: publicKey)
        let expiry = try r.u64()
        let token = try r.take(16)
        let psm = try r.u16()
        let nameLen = Int(try r.u8())
        guard nameLen <= 64 else { throw HandlineError.badLength }
        let nameData = try r.take(nameLen)
        guard r.finished else { throw HandlineError.badLength }
        guard let name = String(data: nameData, encoding: .utf8) else { throw HandlineError.badLength }
        return ParsedQr(deviceId: deviceId, publicKey: publicKey, expiry: expiry, token: token, psm: psm, name: name)
    }

    public static func record(_ kind: UInt8, _ payload: Data) -> Data {
        var out = Data([kind])
        out.appendU16(UInt16(payload.count))
        out.append(payload)
        return out
    }

    public static func clientHello(ephScalar: Data, staticScalar: Data, nonceI: Data) throws -> Data {
        let eph = try P256.KeyAgreement.PrivateKey(rawRepresentation: ephScalar)
        let identity = try P256.KeyAgreement.PrivateKey(rawRepresentation: staticScalar)
        var payload = Data()
        payload.append(eph.publicKey.compressedRepresentation)
        payload.append(identity.publicKey.compressedRepresentation)
        payload.append(nonceI)
        return record(1, payload)
    }

    public static func serverHello(ephScalar: Data, nonceB: Data) throws -> Data {
        let eph = try P256.KeyAgreement.PrivateKey(rawRepresentation: ephScalar)
        var payload = Data()
        payload.append(eph.publicKey.compressedRepresentation)
        payload.append(nonceB)
        return record(2, payload)
    }

    public static func trafficKeys(
        iEph: Data, iStatic: Data, bEph: Data, bStatic: Data,
        nonceI: Data, nonceB: Data, token: Data
    ) throws -> (c2s: Data, s2c: Data) {
        let iEphK = try P256.KeyAgreement.PrivateKey(rawRepresentation: iEph)
        let iStatK = try P256.KeyAgreement.PrivateKey(rawRepresentation: iStatic)
        let bEphP = try P256.KeyAgreement.PublicKey(compressedRepresentation: try compressedPublic(bEph))
        let bStatP = try P256.KeyAgreement.PublicKey(compressedRepresentation: try compressedPublic(bStatic))
        let dh1 = try shared(iEphK, bEphP)
        let dh2 = try shared(iEphK, bStatP)
        let dh3 = try shared(iStatK, bEphP)
        let dh4 = try shared(iStatK, bStatP)
        var ikm = Data()
        ikm.append(dh1); ikm.append(dh2); ikm.append(dh3); ikm.append(dh4)
        var salt = Data()
        salt.append(nonceI); salt.append(nonceB); salt.append(token)
        let okm = HKDF<SHA256>.deriveKey(
            inputKeyMaterial: SymmetricKey(data: ikm),
            salt: salt,
            info: Wire.info,
            outputByteCount: 64
        )
        let bytes = okm.bytes
        return (Data(bytes.prefix(32)), Data(bytes.suffix(32)))
    }

    public static func finished(key: Data, direction: UInt8, clientPayload: Data, serverPayload: Data, token: Data, aad: Data) throws -> Data {
        var transcript = Data()
        transcript.append(clientPayload)
        transcript.append(serverPayload)
        transcript.append(token)
        let hash = Data(SHA256.hash(data: transcript))
        return try seal(key: key, direction: direction, counter: 0, aad: aad, plain: hash)
    }

    public static func seal(key: Data, direction: UInt8, counter: UInt32, aad: Data, plain: Data) throws -> Data {
        let nonce = try AES.GCM.Nonce(data: nonceBytes(direction, counter))
        let box = try AES.GCM.seal(plain, using: SymmetricKey(data: key), nonce: nonce, authenticating: aad)
        return box.ciphertext + box.tag
    }

    public static func open(key: Data, direction: UInt8, counter: UInt32, aad: Data, sealed: Data) throws -> Data {
        guard sealed.count >= 16 else { throw HandlineError.crypto }
        let ct = sealed.prefix(sealed.count - 16)
        let tag = sealed.suffix(16)
        let nonce = try AES.GCM.Nonce(data: nonceBytes(direction, counter))
        let box = try AES.GCM.SealedBox(nonce: nonce, ciphertext: ct, tag: tag)
        return try AES.GCM.open(box, using: SymmetricKey(data: key), authenticating: aad)
    }
}

public struct ReplayWindow {
    public var next: UInt32 = 1
    private var bitmap: UInt64 = 0

    public init() {}

    public mutating func accept(_ counter: UInt32) throws {
        if counter == 0 || counter < next { throw HandlineError.replay }
        let off = counter - next
        if off >= Wire.window { throw HandlineError.counterAhead }
        let bit: UInt64 = 1 << off
        if bitmap & bit != 0 { throw HandlineError.replay }
        bitmap |= bit
        while bitmap & 1 == 1 {
            bitmap >>= 1
            next &+= 1
        }
    }
}

public func frameAAD(type: UInt8, flags: UInt8, counter: UInt32) -> Data {
    var d = Data([1, type, flags])
    d.appendU32(counter)
    return d
}

public func nonceBytes(_ direction: UInt8, _ counter: UInt32) -> Data {
    var n = Data(count: 12)
    n[0] = direction
    let c = counter.bigEndianBytes
    n.replaceSubrange(8..<12, with: c)
    return n
}

private func compressedPublic(_ scalar: Data) throws -> Data {
    try P256.KeyAgreement.PrivateKey(rawRepresentation: scalar).publicKey.compressedRepresentation
}

private func shared(_ priv: P256.KeyAgreement.PrivateKey, _ pub: P256.KeyAgreement.PublicKey) throws -> Data {
    let secret = try priv.sharedSecretFromKeyAgreement(with: pub)
    let bytes = secret.bytes
    guard bytes.count == 32 else { throw HandlineError.crypto }
    return bytes
}

struct Reader {
    var data: Data
    var i = 0
    init(_ data: Data) { self.data = data }
    var finished: Bool { i == data.count }
    mutating func take(_ n: Int) throws -> Data {
        guard i + n <= data.count else { throw HandlineError.truncated }
        let out = data.subdata(in: i..<(i + n))
        i += n
        return out
    }
    mutating func u8() throws -> UInt8 { try take(1)[0] }
    mutating func u16() throws -> UInt16 {
        let b = try take(2)
        return (UInt16(b[0]) << 8) | UInt16(b[1])
    }
    mutating func u64() throws -> UInt64 {
        let b = try take(8)
        var v: UInt64 = 0
        for byte in b { v = (v << 8) | UInt64(byte) }
        return v
    }
}

private extension String {
    func stripPrefix(_ prefix: String) -> String? {
        hasPrefix(prefix) ? String(dropFirst(prefix.count)) : nil
    }
}

private extension Data {
    init?(base64URLEncoded text: String) {
        var s = text.replacingOccurrences(of: "-", with: "+").replacingOccurrences(of: "_", with: "/")
        let pad = (4 - s.count % 4) % 4
        s += String(repeating: "=", count: pad)
        self.init(base64Encoded: s)
    }
    mutating func appendU16(_ v: UInt16) {
        append(UInt8(v >> 8))
        append(UInt8(v & 0xff))
    }
    mutating func appendU32(_ v: UInt32) {
        append(contentsOf: v.bigEndianBytes)
    }
}

private extension UInt32 {
    var bigEndianBytes: Data {
        var be = self.bigEndian
        return Data(bytes: &be, count: 4)
    }
}

private extension SymmetricKey {
    var bytes: Data { withUnsafeBytes { Data($0) } }
}

private extension SharedSecret {
    var bytes: Data { withUnsafeBytes { Data($0) } }
}

