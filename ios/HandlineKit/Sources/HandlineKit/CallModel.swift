// SPDX-License-Identifier: Apache-2.0

import Foundation

public enum CallPhase: Equatable { case ringing, active, held, ended }

public struct HandCall: Equatable {
    public var phase: CallPhase
    public var incoming: Bool
    public var sim: UInt8
    public var digits: String
    public var wentActive: Bool
    public var status: UInt8?
}

public struct CallBook {
    public init() {}
    public private(set) var calls: [UInt64: HandCall] = [:]
    public private(set) var threadSim: [String: UInt8] = [:]
    public private(set) var offHook = false

    public mutating func invite(id: UInt64, incoming: Bool, sim: UInt8, number: String) -> Bool {
        guard calls[id] == nil else { return false }
        let resolved: UInt8 = sim == 0xFF ? (threadSim[number] ?? 0) : sim
        threadSim[number] = resolved
        calls[id] = HandCall(phase: .ringing, incoming: incoming, sim: resolved, digits: "", wentActive: false, status: nil)
        return true
    }

    public mutating func advance(id: UInt64, state: UInt8) -> Bool {
        guard var call = calls[id] else { return false }
        switch (call.phase, state) {
        case (.ringing, 1):
            call.phase = .active
            call.wentActive = true
        case (.held, 1):
            call.phase = .active
        case (.active, 2):
            call.phase = .held
        case (.ringing, 3), (.ringing, 4), (.ringing, 5), (.ringing, 6),
             (.active, 3), (.active, 6), (.held, 3), (.held, 6):
            call.phase = .ended
            call.status = Self.status(call: call, terminal: state)
        default:
            return false
        }
        calls[id] = call
        offHook = calls.values.contains { $0.phase == .active || $0.phase == .held }
        return true
    }

    public mutating func dtmf(id: UInt64, digit: UInt8) -> Bool {
        guard var call = calls[id], call.phase == .active || call.phase == .held else { return false }
        let allowed = Set("0123456789*#ABCD".utf8)
        guard allowed.contains(digit), let scalar = UnicodeScalar(UInt32(digit)) else { return false }
        call.digits.append(Character(scalar))
        calls[id] = call
        return true
    }

    public mutating func dropAll() {
        for id in calls.keys {
            if calls[id]?.phase != .ended {
                _ = advance(id: id, state: 3)
            }
        }
    }

    private static func status(call: HandCall, terminal: UInt8) -> UInt8 {
        switch terminal {
        case 4: return 2
        case 5: return 3
        case 6: return 4
        case 3 where call.wentActive && call.incoming: return 0
        case 3 where call.wentActive && !call.incoming: return 5
        case 3 where call.incoming: return 1
        default: return 4
        }
    }
}

public func isOneTimeCode(_ body: String) -> Bool {
    let trimmed = body.trimmingCharacters(in: .whitespacesAndNewlines)
    if (4...8).contains(trimmed.count), trimmed.allSatisfy(\.isNumber) { return true }
    let lower = body.lowercased()
    let keyword = ["code", "otp", "verify", "verification"].contains { lower.contains($0) }
    return keyword && digitRun(body)
}

private func digitRun(_ body: String) -> Bool {
    var run = 0
    for ch in body {
        if ch.isNumber {
            run += 1
        } else if (4...8).contains(run) {
            return true
        } else {
            run = 0
        }
    }
    return (4...8).contains(run)
}

public enum ProbeFinding: Equatable { case twoWay, uplinkOnly, none }
public enum AudioSource { case voiceCall, voiceCommunication, mic }

public func probeAudio(_ open: (AudioSource) -> Bool) -> ProbeFinding {
    if open(.voiceCall) { return .twoWay }
    if open(.voiceCommunication) || open(.mic) { return .uplinkOnly }
    return .none
}

public enum InstallPhase: Equatable { case idle, pairing, installing, probing, ready, failed }

public struct HelperInstall {
    public private(set) var phase: InstallPhase = .idle
    public private(set) var failure: String?

    public init() {}

    public mutating func submit(code: String) {
        guard code.count == 6, code.allSatisfy(\.isNumber) else {
            failure = "code"
            return
        }
        guard phase == .idle || phase == .failed else { return }
        failure = nil
        phase = .pairing
    }

    public mutating func pairSucceeded() {
        guard phase == .pairing else { return }
        phase = .installing
    }

    public mutating func pairFailed() {
        guard phase == .pairing else { return }
        phase = .failed
        failure = "pair"
    }

    public mutating func installSucceeded() {
        guard phase == .installing else { return }
        phase = .probing
    }

    public mutating func apply(probe: ProbeFinding) {
        guard phase == .probing else { return }
        if probe == .twoWay {
            phase = .ready
        } else {
            phase = .failed
            failure = "audio"
        }
    }
}
