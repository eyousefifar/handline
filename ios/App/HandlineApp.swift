// SPDX-License-Identifier: Apache-2.0
// SwiftUI shell. Compile this with the iOS 17 SDK. The behavior it binds to
// is already tested in ios/HandlineKit.

import SwiftUI

struct HomeView: View {
    var persian: Bool
    var linked: Bool
    var messages: [(String, Bool)]

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(linked ? (persian ? "وصل" : "Linked") : (persian ? "گوشی دیگر در دسترس نیست" : "The other phone is out of reach"))
                .font(.headline)
            ForEach(Array(messages.enumerated()), id: \.offset) { _, row in
                HStack {
                    Text(row.0)
                    if row.1 {
                        Text(persian ? "رمز" : "Code")
                            .font(.caption)
                    }
                }
            }
        }
        .environment(\.layoutDirection, persian ? .rightToLeft : .leftToRight)
        .padding()
    }
}
