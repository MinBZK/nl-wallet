import 'package:bloc_test/bloc_test.dart';
import 'package:mockito/mockito.dart';
import 'package:test/test.dart';
import 'package:wallet/src/data/repository/disclosure/disclosure_repository.dart';
import 'package:wallet/src/feature/disclosure/bloc/disclosure_bloc.dart';
import 'package:wallet/src/feature/report_issue/report_issue_screen.dart';
import 'package:wallet/src/util/extension/bloc_extension.dart';
import 'package:wallet/src/util/extension/string_extension.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../mocks/wallet_mock_data.dart';
import '../../../mocks/wallet_mocks.dart';

void main() {
  late MockStartDisclosureUseCase startDisclosureUseCase;
  late MockCancelDisclosureUseCase cancelDisclosureUseCase;

  /// Create a new [DisclosureBloc] configured with the (mocked) usecases
  DisclosureBloc create() => DisclosureBloc(startDisclosureUseCase, cancelDisclosureUseCase);

  setUp(() {
    startDisclosureUseCase = MockStartDisclosureUseCase();
    cancelDisclosureUseCase = MockCancelDisclosureUseCase();
  });

  test('initial state is correct', () {
    expect(create().state, DisclosureLoadInProgress());
  });

  blocTest(
    'when startDisclosure fails, emit generic error',
    setUp: () => when(startDisclosureUseCase.invoke(any)).thenThrow(Exception('')),
    build: () => create(),
    act: (bloc) => bloc.add(const DisclosureSessionStarted('')),
    expect: () => [DisclosureGenericError()],
  );

  blocTest(
    'when startDisclosure fails with network issue, emit DisclosureNetworkError(hasInternet: true)',
    setUp: () => when(startDisclosureUseCase.invoke(any)).thenThrow(const CoreNetworkError('')),
    build: () => create(),
    act: (bloc) => bloc.add(const DisclosureSessionStarted('')),
    expect: () => [const DisclosureNetworkError(hasInternet: true)],
  );

  blocTest(
    'when startDisclosure fails with network issue and there is no internet, emit DisclosureNetworkError(hasInternet: false)',
    setUp: () {
      when(startDisclosureUseCase.invoke(any)).thenThrow(const CoreNetworkError(''));
      when(BlocExtensions.checkHasInternetUseCase.invoke()).thenAnswer((realInvocation) async => false);
    },
    build: () => create(),
    act: (bloc) => bloc.add(const DisclosureSessionStarted('')),
    expect: () => [const DisclosureNetworkError(hasInternet: false)],
  );

  blocTest(
    'when startDisclosure returns StartDisclosureReadyToDisclose, the bloc emits DisclosureCheckOrganization',
    setUp: () {
      when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
        return StartDisclosureReadyToDisclose(
          WalletMockData.organization,
          WalletMockData.policy,
          'requestPurpose'.untranslated,
          'http://origin.org',
          false,
          {},
        );
      });
    },
    build: () => create(),
    act: (bloc) => bloc.add(const DisclosureSessionStarted('')),
    expect: () => [isA<DisclosureCheckOrganization>()],
  );

  blocTest(
    'when startDisclosure returns StartDisclosureMissingAttributes, the bloc emits DisclosureCheckOrganization',
    setUp: () {
      when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
        return StartDisclosureMissingAttributes(
          WalletMockData.organization,
          'requestPurpose'.untranslated,
          'http://origin.org',
          false,
          [],
        );
      });
    },
    build: () => create(),
    act: (bloc) => bloc.add(const DisclosureSessionStarted('')),
    expect: () => [isA<DisclosureCheckOrganization>()],
  );

  blocTest(
    'when the user stops disclosure while checking the organization for ready to disclose, the bloc emits DisclosureStopped and cancels disclosure',
    setUp: () {
      when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
        return StartDisclosureReadyToDisclose(
          WalletMockData.organization,
          WalletMockData.policy,
          'requestPurpose'.untranslated,
          'http://origin.org',
          false,
          {},
        );
      });
    },
    build: () => create(),
    act: (bloc) {
      bloc.add(const DisclosureSessionStarted(''));
      bloc.add(const DisclosureStopRequested());
    },
    expect: () => [isA<DisclosureCheckOrganization>(), isA<DisclosureLoadInProgress>(), isA<DisclosureStopped>()],
    verify: (bloc) => verify(cancelDisclosureUseCase.invoke()).called(1),
  );

  blocTest(
    'when the user stops disclosure while checking the organization for missing attributes, the bloc emits DisclosureStopped and cancels disclosure',
    setUp: () {
      when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
        return StartDisclosureMissingAttributes(
          WalletMockData.organization,
          'requestPurpose'.untranslated,
          'http://origin.org',
          false,
          [],
        );
      });
    },
    build: () => create(),
    act: (bloc) {
      bloc.add(const DisclosureSessionStarted(''));
      bloc.add(const DisclosureStopRequested());
    },
    expect: () => [isA<DisclosureCheckOrganization>(), isA<DisclosureLoadInProgress>(), isA<DisclosureStopped>()],
    verify: (bloc) => verify(cancelDisclosureUseCase.invoke()).called(1),
  );

  blocTest(
    'when the user continues disclosure while checking the organization for ready to disclose, the bloc emits DisclosureConfirmDataAttributes',
    setUp: () {
      when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
        return StartDisclosureReadyToDisclose(
          WalletMockData.organization,
          WalletMockData.policy,
          'requestPurpose'.untranslated,
          'http://origin.org',
          false,
          {},
        );
      });
    },
    build: () => create(),
    act: (bloc) async {
      bloc.add(const DisclosureSessionStarted(''));
      bloc.add(const DisclosureOrganizationApproved());
    },
    expect: () => [isA<DisclosureCheckOrganization>(), isA<DisclosureConfirmDataAttributes>()],
  );

  blocTest(
    'when the user continues disclosure while checking the organization for missing attributes, the bloc emits DisclosureMissingAttributes',
    setUp: () {
      when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
        return StartDisclosureMissingAttributes(
          WalletMockData.organization,
          'requestPurpose'.untranslated,
          'http://origin.org',
          false,
          [],
        );
      });
    },
    build: () => create(),
    act: (bloc) async {
      bloc.add(const DisclosureSessionStarted(''));
      bloc.add(const DisclosureOrganizationApproved());
    },
    expect: () => [isA<DisclosureCheckOrganization>(), isA<DisclosureMissingAttributes>()],
  );

  blocTest(
    'when users stops the flow reviewing the DisclosureMissingAttributes state, the bloc emits DisclosureLoadInProgress and DisclosureStopped states and cancels disclosure',
    setUp: () {
      when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
        return StartDisclosureMissingAttributes(
          WalletMockData.organization,
          'requestPurpose'.untranslated,
          'http://origin.org',
          false,
          [],
        );
      });
    },
    build: () => create(),
    act: (bloc) async {
      bloc.add(const DisclosureSessionStarted(''));
      bloc.add(const DisclosureOrganizationApproved());
      bloc.add(const DisclosureStopRequested());
    },
    verify: (bloc) => verify(cancelDisclosureUseCase.invoke()).called(1),
    expect: () => [
      isA<DisclosureCheckOrganization>(),
      isA<DisclosureMissingAttributes>(),
      isA<DisclosureLoadInProgress>(),
      isA<DisclosureStopped>(),
    ],
  );

  blocTest(
    'when the user opts so share the requested attributes, the bloc emits DisclosureConfirmPin',
    setUp: () {
      when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
        return StartDisclosureReadyToDisclose(
          WalletMockData.organization,
          WalletMockData.policy,
          'requestPurpose'.untranslated,
          'http://origin.org',
          false,
          {},
        );
      });
    },
    build: () => create(),
    act: (bloc) async {
      bloc.add(const DisclosureSessionStarted(''));
      bloc.add(const DisclosureOrganizationApproved());
      bloc.add(const DisclosureShareRequestedAttributesApproved());
    },
    skip: 2,
    expect: () => [isA<DisclosureConfirmPin>()],
  );

  blocTest(
    'when the user confirms the pin, the bloc emits DisclosureSuccess',
    setUp: () {
      when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
        return StartDisclosureReadyToDisclose(
          WalletMockData.organization,
          WalletMockData.policy,
          'requestPurpose'.untranslated,
          'http://origin.org',
          false,
          {},
        );
      });
    },
    build: () => create(),
    act: (bloc) async {
      bloc.add(const DisclosureSessionStarted(''));
      bloc.add(const DisclosureOrganizationApproved());
      bloc.add(const DisclosureShareRequestedAttributesApproved());
      bloc.add(const DisclosurePinConfirmed());
    },
    skip: 3,
    expect: () => [isA<DisclosureSuccess>()],
  );

  blocTest(
    'when the user leaves feedback when stopping, the bloc emits DisclosureLeftFeedback and disclosure is cancelled',
    setUp: () {
      when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
        return StartDisclosureReadyToDisclose(
          WalletMockData.organization,
          WalletMockData.policy,
          'requestPurpose'.untranslated,
          'http://origin.org',
          false,
          {},
        );
      });
    },
    build: () => create(),
    act: (bloc) async {
      bloc.add(const DisclosureSessionStarted(''));
      bloc.add(const DisclosureReportPressed(option: ReportingOption.impersonatingOrganization));
    },
    verify: (bloc) => verify(cancelDisclosureUseCase.invoke()).called(1),
    expect: () => [
      isA<DisclosureCheckOrganization>(),
      isA<DisclosureLoadInProgress>(),
      isA<DisclosureLeftFeedback>(),
    ],
  );

  blocTest(
    'when user presses back from the DisclosureCheckOrganization state, the bloc emits DisclosureCheckOrganization ',
    setUp: () {
      when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
        return StartDisclosureReadyToDisclose(
          WalletMockData.organization,
          WalletMockData.policy,
          'requestPurpose'.untranslated,
          'http://origin.org',
          false,
          {},
        );
      });
    },
    build: () => create(),
    act: (bloc) async {
      bloc.add(const DisclosureSessionStarted(''));
      bloc.add(const DisclosureOrganizationApproved());
      bloc.add(const DisclosureBackPressed());
    },
    expect: () => [
      isA<DisclosureCheckOrganization>(),
      isA<DisclosureConfirmDataAttributes>(),
      isA<DisclosureCheckOrganization>(),
    ],
  );

  blocTest(
    'when user presses back from the DisclosureConfirmPin state, the bloc emits DisclosureConfirmDataAttributes ',
    setUp: () {
      when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
        return StartDisclosureReadyToDisclose(
          WalletMockData.organization,
          WalletMockData.policy,
          'requestPurpose'.untranslated,
          'http://origin.org',
          false,
          {},
        );
      });
    },
    build: () => create(),
    act: (bloc) async {
      bloc.add(const DisclosureSessionStarted(''));
      bloc.add(const DisclosureOrganizationApproved());
      bloc.add(const DisclosureShareRequestedAttributesApproved());
      bloc.add(const DisclosureBackPressed());
    },
    skip: 1,
    expect: () => [
      isA<DisclosureConfirmDataAttributes>(),
      isA<DisclosureConfirmPin>(),
      isA<DisclosureConfirmDataAttributes>(),
    ],
  );

  blocTest(
    'when user presses back from the DisclosureMissingAttributes state, the bloc emits DisclosureCheckOrganization ',
    setUp: () {
      when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
        return StartDisclosureMissingAttributes(
          WalletMockData.organization,
          'requestPurpose'.untranslated,
          'http://origin.org',
          false,
          [],
        );
      });
    },
    build: () => create(),
    act: (bloc) async {
      bloc.add(const DisclosureSessionStarted(''));
      bloc.add(const DisclosureOrganizationApproved());
      bloc.add(const DisclosureBackPressed());
    },
    expect: () => [
      isA<DisclosureCheckOrganization>(),
      isA<DisclosureMissingAttributes>(),
      isA<DisclosureCheckOrganization>(),
    ],
  );
}
