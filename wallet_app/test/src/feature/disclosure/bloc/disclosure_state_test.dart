import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/disclosure/disclose_card_request.dart';
import 'package:wallet/src/domain/model/disclosure/disclosure_session_type.dart';
import 'package:wallet/src/domain/model/flow_progress.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/feature/disclosure/bloc/disclosure_bloc.dart';
import 'package:wallet/src/util/extension/string_extension.dart';

import '../../../mocks/wallet_mock_data.dart';

void main() {
  group('DisclosureState FlowProgress Tests', () {
    final mockApplicationError = const GenericError('test', sourceError: 'test');
    final mockOrganization = WalletMockData.organization;
    final mockLocalizedText = 'Localized Text'.untranslated;
    final mockCardRequests = <DiscloseCardRequest>[];
    final mockPolicy = WalletMockData.policy;
    final mockMissingAttributes = <MissingAttribute>[];

    test('DisclosureInitial has correct default FlowProgress', () {
      const state = DisclosureInitial();
      expect(state.stepperProgress, const FlowProgress(currentStep: 0, totalSteps: kDisclosureSteps));
    });

    test('DisclosureLoadInProgress has correct FlowProgress', () {
      const progress = FlowProgress(currentStep: 1, totalSteps: kDisclosureSteps);
      const state = DisclosureLoadInProgress(progress);
      expect(state.stepperProgress, progress);
    });

    test('DisclosureCheckUrl has correct FlowProgress', () {
      const state = DisclosureCheckUrl(originUrl: 'https://example.com');
      expect(
        state.stepperProgress,
        const FlowProgress(currentStep: 1, totalSteps: kDisclosureSteps + kExtraCrossDeviceSteps),
      );
    });

    group('DisclosureCheckOrganizationForLogin FlowProgress', () {
      test('is correct for same device flow', () {
        final state = DisclosureCheckOrganizationForLogin(
          relyingParty: mockOrganization,
          originUrl: 'https://example.com',
          sessionType: DisclosureSessionType.sameDevice,
          policy: mockPolicy,
          cardRequests: mockCardRequests,
          sharedDataWithOrganizationBefore: false,
        );
        expect(state.stepperProgress, const FlowProgress(currentStep: 1, totalSteps: kDisclosureSteps));
      });

      test('is correct for cross device flow', () {
        final state = DisclosureCheckOrganizationForLogin(
          relyingParty: mockOrganization,
          originUrl: 'https://example.com',
          sessionType: DisclosureSessionType.crossDevice,
          policy: mockPolicy,
          cardRequests: mockCardRequests,
          sharedDataWithOrganizationBefore: false,
        );
        expect(
          state.stepperProgress,
          const FlowProgress(currentStep: 2, totalSteps: kDisclosureSteps + kExtraCrossDeviceSteps),
        );
      });
    });

    group('DisclosureMissingAttributes FlowProgress', () {
      test('is correct for same device flow', () {
        final state = DisclosureMissingAttributes(
          relyingParty: mockOrganization,
          missingAttributes: mockMissingAttributes,
          isCrossDevice: false,
        );
        expect(state.stepperProgress, const FlowProgress(currentStep: 1, totalSteps: kDisclosureSteps));
      });

      test('is correct for cross device flow', () {
        final state = DisclosureMissingAttributes(
          relyingParty: mockOrganization,
          missingAttributes: mockMissingAttributes,
          isCrossDevice: true,
        );
        expect(
          state.stepperProgress,
          const FlowProgress(currentStep: 2, totalSteps: kDisclosureSteps + kExtraCrossDeviceSteps),
        );
      });
    });

    group('DisclosureConfirmDataAttributes FlowProgress', () {
      test('is correct for same device flow', () {
        final state = DisclosureConfirmDataAttributes(
          relyingParty: mockOrganization,
          requestPurpose: mockLocalizedText,
          cardRequests: mockCardRequests,
          policy: mockPolicy,
          sessionType: DisclosureSessionType.sameDevice,
        );
        expect(state.stepperProgress, const FlowProgress(currentStep: 1, totalSteps: kDisclosureSteps));
      });

      test('is correct for cross device flow', () {
        final state = DisclosureConfirmDataAttributes(
          relyingParty: mockOrganization,
          requestPurpose: mockLocalizedText,
          cardRequests: mockCardRequests,
          policy: mockPolicy,
          sessionType: DisclosureSessionType.crossDevice,
        );
        expect(
          state.stepperProgress,
          const FlowProgress(currentStep: 2, totalSteps: kDisclosureSteps + kExtraCrossDeviceSteps),
        );
      });
    });

    group('DisclosureConfirmPin FlowProgress', () {
      test('is correct for same device flow', () {
        final state = DisclosureConfirmPin(
          relyingParty: mockOrganization,
          isCrossDevice: false,
        );
        expect(state.stepperProgress, const FlowProgress(currentStep: 2, totalSteps: kDisclosureSteps));
      });

      test('is correct for cross device flow', () {
        final state = DisclosureConfirmPin(
          relyingParty: mockOrganization,
          isCrossDevice: true,
        );
        expect(
          state.stepperProgress,
          const FlowProgress(
            currentStep: 2 + kExtraCrossDeviceSteps,
            totalSteps: kDisclosureSteps + kExtraCrossDeviceSteps,
          ),
        );
      });
    });

    group('DisclosureSuccess FlowProgress', () {
      test('is correct for same device flow', () {
        final state = DisclosureSuccess(
          relyingParty: mockOrganization,
          isCrossDevice: false,
        );
        expect(state.stepperProgress, const FlowProgress(currentStep: kDisclosureSteps, totalSteps: kDisclosureSteps));
      });

      test('is correct for cross device flow', () {
        final state = DisclosureSuccess(
          relyingParty: mockOrganization,
          isCrossDevice: true,
        );
        expect(
          state.stepperProgress,
          const FlowProgress(
            currentStep: kDisclosureSteps + kExtraCrossDeviceSteps,
            totalSteps: kDisclosureSteps + kExtraCrossDeviceSteps,
          ),
        );
      });
    });

    test('DisclosureStopped has correct FlowProgress', () {
      final state = DisclosureStopped(organization: mockOrganization);
      expect(state.stepperProgress, const FlowProgress(currentStep: kDisclosureSteps, totalSteps: kDisclosureSteps));
    });

    test('DisclosureLeftFeedback has correct FlowProgress', () {
      const state = DisclosureLeftFeedback();
      expect(state.stepperProgress, const FlowProgress(currentStep: kDisclosureSteps, totalSteps: kDisclosureSteps));
    });

    group('Disclosure ErrorStates', () {
      test('DisclosureExternalScannerError has correct default FlowProgress', () {
        final state = DisclosureExternalScannerError(error: mockApplicationError);
        expect(state.stepperProgress, const FlowProgress(currentStep: 0, totalSteps: kDisclosureSteps));
      });

      test('DisclosureGenericError has correct default FlowProgress', () {
        final state = DisclosureGenericError(error: mockApplicationError);
        expect(state.stepperProgress, const FlowProgress(currentStep: 0, totalSteps: kDisclosureSteps));
      });

      test('DisclosureRelyingPartyError has correct default FlowProgress', () {
        final state = DisclosureRelyingPartyError(error: mockApplicationError);
        expect(state.stepperProgress, const FlowProgress(currentStep: 0, totalSteps: kDisclosureSteps));
      });

      test('DisclosureSessionExpired has correct default FlowProgress', () {
        final state = DisclosureSessionExpired(error: mockApplicationError, isCrossDevice: false, canRetry: false);
        expect(state.stepperProgress, const FlowProgress(currentStep: 0, totalSteps: kDisclosureSteps));
      });

      test('DisclosureSessionCancelled has correct default FlowProgress', () {
        final state = DisclosureSessionCancelled(error: mockApplicationError);
        expect(state.stepperProgress, const FlowProgress(currentStep: 0, totalSteps: kDisclosureSteps));
      });

      test('DisclosureNetworkError has correct default FlowProgress', () {
        final state = DisclosureNetworkError(error: mockApplicationError);
        expect(state.stepperProgress, const FlowProgress(currentStep: 0, totalSteps: kDisclosureSteps));
      });
    });
  });
}
