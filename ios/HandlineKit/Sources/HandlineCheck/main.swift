// SPDX-License-Identifier: Apache-2.0

import Foundation
import HandlineKit

var failed = 0
func check(_ condition: Bool, _ note: String) {
    if !condition {
        failed += 1
        FileHandle.standardError.write(Data("FAIL \(note)\n".utf8))
    }
}

let root = URL(fileURLWithPath: #filePath)
    .deletingLastPathComponent()
    .deletingLastPathComponent()
    .deletingLastPathComponent()
    .deletingLastPathComponent()
    .deletingLastPathComponent()
let url = root.appendingPathComponent("handline-core/testdata/reference.json")
let raw = try Data(contentsOf: url)
let json = try JSONSerialization.jsonObject(with: raw) as! [String: Any]
func hex(_ key: String) -> Data { dataFromHex(json[key] as! String) }
func text(_ key: String) -> String { json[key] as! String }

let qr = try HandlineCodec.decodeQr(text("qr"))
check(qr.name == "Bag", "qr name")
check(qr.psm == 79, "psm")
check(qr.token == hex("token_hex"), "token")
check(qr.expiry == UInt64(json["now"] as! Int) + 300, "expiry")

let ch = try HandlineCodec.clientHello(
    ephScalar: hex("hand_eph_scalar_hex"),
    staticScalar: hex("hand_scalar_hex"),
    nonceI: hex("nonce_i_hex")
)
check(ch == hex("client_hello_hex"), "client hello")
let sh = try HandlineCodec.serverHello(ephScalar: hex("bag_eph_scalar_hex"), nonceB: hex("nonce_b_hex"))
check(sh == hex("server_hello_hex"), "server hello")

let keys = try HandlineCodec.trafficKeys(
    iEph: hex("hand_eph_scalar_hex"),
    iStatic: hex("hand_scalar_hex"),
    bEph: hex("bag_eph_scalar_hex"),
    bStatic: hex("bag_scalar_hex"),
    nonceI: hex("nonce_i_hex"),
    nonceB: hex("nonce_b_hex"),
    token: hex("token_hex")
)
check(keys.c2s == hex("c2s_key_hex"), "c2s")
check(keys.s2c == hex("s2c_key_hex"), "s2c")

let cf = try HandlineCodec.finished(
    key: keys.c2s, direction: 1,
    clientPayload: Data(ch.dropFirst(3)), serverPayload: Data(sh.dropFirst(3)),
    token: hex("token_hex"), aad: Data("cfin".utf8)
)
check(HandlineCodec.record(3, cf) == hex("client_finished_hex"), "client finished")
let sf = try HandlineCodec.finished(
    key: keys.s2c, direction: 2,
    clientPayload: Data(ch.dropFirst(3)), serverPayload: Data(sh.dropFirst(3)),
    token: hex("token_hex"), aad: Data("sfin".utf8)
)
check(HandlineCodec.record(4, sf) == hex("server_finished_hex"), "server finished")

let hello = hex("hello_hand_hex")
let plain = try openFrame(hello, key: keys.c2s, direction: 1)
check(plain[0] == 1 && plain[1] == 1, "hello body")
var window = ReplayWindow()
try window.accept(counter(of: hello))
do {
    try window.accept(counter(of: hello))
    check(false, "replay should throw")
} catch {
    check(true, "replay")
}
let tampered = hex("tampered_ping_hex")
do {
    _ = try HandlineCodec.open(
        key: keys.c2s, direction: 1, counter: counter(of: tampered),
        aad: frameAAD(type: tampered[1], flags: tampered[2], counter: counter(of: tampered)),
        sealed: sealed(of: tampered)
    )
    check(false, "tamper should throw")
} catch {
    check(true, "tamper")
}

var book = CallBook()
check(book.invite(id: 7, incoming: true, sim: 0, number: "+1"), "invite")
check(book.advance(id: 7, state: 1) && book.offHook, "answer")
check(book.dtmf(id: 7, digit: UInt8(ascii: "5")), "dtmf")
check(book.calls[7]?.digits == "5", "digits")
check(book.advance(id: 7, state: 3) && !book.offHook, "hangup")
check(book.calls[7]?.status == 0, "answered status")

var missed = CallBook()
_ = missed.invite(id: 4, incoming: true, sim: 0, number: "+1")
_ = missed.advance(id: 4, state: 3)
check(missed.calls[4]?.status == 1, "missed")

var sims = CallBook()
_ = sims.invite(id: 1, incoming: false, sim: 0, number: "+200")
_ = sims.invite(id: 2, incoming: false, sim: 0xFF, number: "+200")
_ = sims.invite(id: 3, incoming: false, sim: 1, number: "+200")
_ = sims.invite(id: 4, incoming: false, sim: 0xFF, number: "+200")
check(sims.calls[4]?.sim == 1, "sim memory")

check(isOneTimeCode("123456"), "otp digits")
check(isOneTimeCode("Your code is 482193"), "otp words")
check(!isOneTimeCode("Lunch at 1234 Main"), "not otp")
check(probeAudio { $0 == .voiceCall } == .twoWay, "probe two way")
check(probeAudio { $0 == .mic } == .uplinkOnly, "probe mic")
check(probeAudio { _ in false } == .none, "probe none")

var install = HelperInstall()
install.submit(code: "12")
check(install.phase == .idle, "short code")
install.submit(code: "123456")
install.installSucceeded()
check(install.phase == .pairing, "no install before pair")
install.pairSucceeded()
install.installSucceeded()
install.apply(probe: .uplinkOnly)
check(install.phase == .failed, "uplink is not tier b")
install.submit(code: "654321")
install.pairSucceeded()
install.installSucceeded()
install.apply(probe: .twoWay)
check(install.phase == .ready, "tier b ready")

if failed > 0 {
    FileHandle.standardError.write(Data("\(failed) checks failed\n".utf8))
    exit(1)
}
print("handline swift checks passed")

func dataFromHex(_ hex: String) -> Data {
    var out = Data()
    var i = hex.startIndex
    while i < hex.endIndex {
        let j = hex.index(i, offsetBy: 2)
        out.append(UInt8(hex[i..<j], radix: 16)!)
        i = j
    }
    return out
}
func counter(of frame: Data) -> UInt32 {
    (UInt32(frame[3]) << 24) | (UInt32(frame[4]) << 16) | (UInt32(frame[5]) << 8) | UInt32(frame[6])
}
func sealed(of frame: Data) -> Data {
    let ctLen = Int(frame[7]) << 8 | Int(frame[8])
    return frame.subdata(in: 9..<(9 + ctLen + 16))
}
func openFrame(_ frame: Data, key: Data, direction: UInt8) throws -> Data {
    try HandlineCodec.open(
        key: key, direction: direction, counter: counter(of: frame),
        aad: frameAAD(type: frame[1], flags: frame[2], counter: counter(of: frame)),
        sealed: sealed(of: frame)
    )
}
