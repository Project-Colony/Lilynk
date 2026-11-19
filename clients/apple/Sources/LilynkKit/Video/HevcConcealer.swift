// HEVC reference concealment for VideoToolbox (`lilynk_h265_concealer_*`, ABI v29).
//
// VideoToolbox reads the reference picture set itself, and one current entry it does not hold
// puts the session into an error state that refuses every later non-IDR access unit, the RFI
// anchor and every intra-refresh wave frame included. The core's concealer mirrors the
// decoder's DPB and, when an AU names a lost picture as current, rewrites its slice headers so
// a present picture stands in. Feed it exactly what the decoder gets, in order.

import Foundation
import LilynkCore

final class HevcConcealer {
    enum Outcome {
        /// Decode the AU as it came.
        case intact
        /// Decode these bytes instead.
        case rewritten(Data)
        /// Nothing can stand in: keep this AU off the decoder until the host re-anchors.
        case unrecoverable
    }

    private let ptr: OpaquePointer

    init() {
        ptr = lilynk_h265_concealer_new()
    }

    deinit { lilynk_h265_concealer_free(ptr) }

    func conceal(_ au: Data) -> Outcome {
        var kind = LILYNK_CONCEALMENT_INTACT
        var buf: UnsafeMutablePointer<UInt8>?
        var len: UInt = 0
        let status = au.withUnsafeBytes { raw in
            lilynk_h265_concealer_conceal(
                ptr, raw.bindMemory(to: UInt8.self).baseAddress, UInt(raw.count), &kind, &buf,
                &len)
        }
        guard status == LILYNK_STATUS_OK.rawValue else { return .intact }
        switch kind.rawValue {
        case LILYNK_CONCEALMENT_REWRITTEN.rawValue:
            guard let buf else { return .intact }
            let data = Data(bytes: buf, count: Int(len))
            lilynk_h265_concealer_release(buf, len)
            return .rewritten(data)
        case LILYNK_CONCEALMENT_UNRECOVERABLE.rawValue:
            return .unrecoverable
        default:
            return .intact
        }
    }
}
