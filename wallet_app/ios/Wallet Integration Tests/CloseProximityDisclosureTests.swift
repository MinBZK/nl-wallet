import Foundation
import XCTest

import PlatformSupport

final class CloseProximityDisclosureTests: XCTestCase {
    override class func setUp() {
        _ = PlatformSupport.shared
    }

    func testWalletCloseProximityDisclosureWithMacReader() async throws {
        await runWalletDisclosure()
    }

    func testWalletJwtRegistrationCertificateWithMacReader() async {
        await runWalletDisclosure(jwtRegistrationCertificate: true)
    }

    func testWalletRegistrationCertificateMissing() async {
        await runWalletDisclosure(registrationCase: "missing")
    }

    func testWalletRegistrationCertificateWrongType() async {
        await runWalletDisclosure(registrationCase: "wrong_type")
    }

    func testWalletRegistrationCertificateMalformed() async {
        await runWalletDisclosure(registrationCase: "malformed")
    }

    func testWalletRegistrationCertificateTamperedSignature() async {
        await runWalletDisclosure(registrationCase: "tampered_signature")
    }

    func testWalletRegistrationCertificateExpired() async {
        await runWalletDisclosure(registrationCase: "expired")
    }

    func testWalletRegistrationCertificateUntrusted() async {
        await runWalletDisclosure(registrationCase: "untrusted")
    }

    func testWalletRegistrationCertificateMismatchedReader() async {
        await runWalletDisclosure(registrationCase: "mismatched_reader")
    }

    func testWalletRegistrationCertificateMismatchedSubjectType() async {
        await runWalletDisclosure(registrationCase: "mismatched_subject_type")
    }

    func testWalletRegistrationCertificateRevoked() async {
        await runWalletDisclosure(registrationCase: "revoked")
    }

    func testWalletRegistrationCertificateSuspended() async {
        await runWalletDisclosure(registrationCase: "suspended")
    }

    func testWalletRegistrationCertificateStatusUnavailable() async {
        await runWalletDisclosure(registrationCase: "status_unavailable")
    }

    func testWalletRegistrationCertificateUntrustedStatusList() async {
        await runWalletDisclosure(registrationCase: "untrusted_status_list")
    }

    func testWalletRegistrationCertificateStatusIndexOutOfBounds() async {
        await runWalletDisclosure(registrationCase: "status_index_out_of_bounds")
    }

    func testWalletRegistrationCertificateUnauthorizedDocumentType() async {
        await runWalletDisclosure(registrationCase: "unauthorized_document_type")
    }

    func testWalletRegistrationCertificateUnauthorizedNamespace() async {
        await runWalletDisclosure(registrationCase: "unauthorized_namespace")
    }

    func testWalletRegistrationCertificateUnauthorizedAttribute() async {
        await runWalletDisclosure(registrationCase: "unauthorized_attribute")
    }

    func testWalletRegistrationCertificateUnauthorizedSecondDocument() async {
        await runWalletDisclosure(registrationCase: "unauthorized_second_document")
    }

    private func runWalletDisclosure(
        registrationCase: String? = nil,
        jwtRegistrationCertificate: Bool = false,
        file: StaticString = #filePath,
        line: UInt = #line
    ) async {
        // The main thread must remain available for CoreBluetooth callbacks while Rust runs Tokio.
        let error: String? = await withCheckedContinuation { continuation in
            let thread = Thread {
                let readerReady: @convention(c) (UnsafePointer<CChar>?) -> Void = { marker in
                    guard let marker else { return }
                    // Signed request fixtures exceed unified logging's message-size limits.
                    FileHandle.standardError.write(Data((String(cString: marker) + "\n").utf8))
                }
                let error: UnsafeMutablePointer<CChar>?
                if let registrationCase {
                    error = registrationCase.withCString { wallet_registration_certificate_test($0, readerReady) }
                } else if jwtRegistrationCertificate {
                    error = wallet_jwt_registration_certificate_test(readerReady)
                } else {
                    error = wallet_close_proximity_disclosure_test(readerReady)
                }
                guard let error else {
                    continuation.resume(returning: nil)
                    return
                }
                let message = String(cString: error)
                wallet_device_test_free_error(error)
                continuation.resume(returning: message)
            }
            // Debug-build compression needs more stack than Swift concurrency workers provide.
            thread.stackSize = 2 * 1024 * 1024
            thread.start()
        }
        XCTAssertNil(error, error ?? "", file: file, line: line)
    }
}
