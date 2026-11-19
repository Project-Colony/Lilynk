// The video-capability byte, which is a wire promise: every bit says the decoder and the
// present path can take what the host is then free to send.
//
// Depth and HDR are separate asks. The pair that must not drift is "HDR without the depth":
// the host would upgrade the stream and hand the decoder Main10 the client never claimed.

import XCTest

@testable import LilynkKit

final class VideoCapsTests: XCTestCase {
    func testDepthAndHDRAreSeparateAsks() {
        XCTAssertEqual(
            LilynkConnection.videoCaps(tenBit: false, hdr: false, chroma444: false), 0,
            "nothing advertised is the 8-bit BT.709 SDR stream")

        // `ten_bit_sdr`: Main10 for a desktop, and no HDR promise with it.
        XCTAssertEqual(
            LilynkConnection.videoCaps(tenBit: true, hdr: false, chroma444: false),
            LilynkConnection.videoCap10Bit)

        // HDR carries the depth even when the ten-bit ask was off — the two cannot separate
        // in that direction, because HDR10 is Main10.
        XCTAssertEqual(
            LilynkConnection.videoCaps(tenBit: false, hdr: true, chroma444: false),
            LilynkConnection.videoCap10Bit | LilynkConnection.videoCapHDR)

        XCTAssertEqual(
            LilynkConnection.videoCaps(tenBit: true, hdr: true, chroma444: true),
            LilynkConnection.videoCap10Bit | LilynkConnection.videoCapHDR
                | LilynkConnection.videoCap444)

        // 4:4:4 rides on whatever depth was asked for, and never implies one.
        XCTAssertEqual(
            LilynkConnection.videoCaps(tenBit: false, hdr: false, chroma444: true),
            LilynkConnection.videoCap444)
    }
}
