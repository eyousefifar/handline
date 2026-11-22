# iOS

iOS 17 or later. The tested code is the Swift package `ios/HandlineKit`. SwiftUI screens in `ios/App` need the iOS SDK from a full Xcode install. This Mac had Command Line Tools only while the package tests were written, so those screens are not compiled here. `swift test` covers the protocol fixture, calls, SIMs, codes, and the helper state machine.

## Call UI

`CallBook` is the source of truth. A presenter is told to show, update, and end a call.

| Book event | CallKit adapter |
| --- | --- |
| incoming invite | report a new incoming call, including on the lock screen when iOS has woken the app |
| active | fulfill the answer action |
| ended or declined | end the call with the matching reason |
| out of range | end live calls and report the bag phone as unavailable |

The adapter is a thin map. Tests use a fake presenter and assert the calls it received. The package does not import CallKit, so the tests run on macOS.

Background Bluetooth is `bluetooth-central` plus state restoration. iOS may suspend the process. The UI must say the bag phone is asleep or out of range instead of claiming a call rang when the app was not running. That limit is part of the product, not a bug to hide.

## Storage

Messages, contacts, and history live in the app. The tested book never calls `CNContactStore`. A contact added on the iPhone is a protocol message to the bag phone, not an insert into the iPhone address book.

## Language

`Copy.string` returns English or Persian. Persian is right to left. The strings are original.
