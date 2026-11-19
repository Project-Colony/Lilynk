// The Controller type slot's marks. Its step order and words replay the shared vectors in
// `OverlayActionsTests.testSharedPadTypeCycle`.

import XCTest
@testable import LilynkKit

final class PadTypeTests: XCTestCase {
    /// A name with no imageset draws nothing at runtime, so check the catalog itself.
    func testEveryPickedTypeShipsAMark() {
        let catalog = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
            .appendingPathComponent("Sources/LilynkKit/Resources/PadMarks.xcassets")
        for type in LilynkConnection.GamepadType.ringCycle where type != .auto {
            guard let name = type.markName else { return XCTFail("\(type) has no outline") }
            let pdf = catalog.appendingPathComponent("\(name).imageset/\(name).pdf")
            XCTAssertTrue(FileManager.default.fileExists(atPath: pdf.path), "\(name).pdf missing")
        }
        XCTAssertNil(LilynkConnection.GamepadType.auto.mark)
    }
}
