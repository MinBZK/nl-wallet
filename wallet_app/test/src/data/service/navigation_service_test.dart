import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/service/navigation_service.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';
import 'package:wallet/src/feature/disclosure/argument/disclosure_screen_argument.dart';
import 'package:wallet/src/feature/issuance/argument/issuance_screen_argument.dart';
import 'package:wallet/src/feature/sign/argument/sign_screen_argument.dart';

import '../../mocks/wallet_mocks.dart';

void main() {
  late NavigationService service;
  late MockCheckNavigationPrerequisitesUseCase mockCheckNavigationPrerequisitesUseCase;
  late MockPerformPreNavigationActionsUseCase mockPerformPreNavigationActionsUseCase;
  late MockGetWalletStateUseCase mockGetWalletStateUseCase;
  late MockNavigatorKey navigatorKey;
  late MockNavigatorState navigatorState;
  late MockBuildContext context;

  setUp(() {
    navigatorKey = MockNavigatorKey();
    navigatorState = MockNavigatorState();
    context = MockBuildContext();
    when(navigatorState.context).thenAnswer((_) => context);
    when(navigatorKey.currentState).thenReturn(navigatorState);
    // Usecases
    mockCheckNavigationPrerequisitesUseCase = MockCheckNavigationPrerequisitesUseCase();
    mockPerformPreNavigationActionsUseCase = MockPerformPreNavigationActionsUseCase();
    mockGetWalletStateUseCase = MockGetWalletStateUseCase();

    service = NavigationService(
      navigatorKey,
      mockCheckNavigationPrerequisitesUseCase,
      mockPerformPreNavigationActionsUseCase,
      mockGetWalletStateUseCase,
    );
  });

  group('handleNavigationRequest', () {
    test('When navigation prerequisites are fulfilled, trigger actual navigation', () async {
      // Allow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);

      const navigationRequest = GenericNavigationRequest('/mock');
      await service.handleNavigationRequest(navigationRequest);

      verify(navigatorState.pushNamed(navigationRequest.destination));
    });

    test('Verify PidIssuanceNavigationRequest triggers navigation', () async {
      // Allow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);

      final navigationRequest = NavigationRequest.pidIssuance('/mock');
      await service.handleNavigationRequest(navigationRequest);

      verify(
        navigatorState.pushNamedAndRemoveUntil(
          navigationRequest.destination,
          any,
          arguments: anyNamed('arguments'),
        ),
      );
    });

    test('Verify DisclosureNavigationRequest triggers navigation', () async {
      // Allow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);

      final navigationRequest = NavigationRequest.disclosure(
        argument: const DisclosureScreenArgument(uri: '/mock', isQrCode: false),
      );
      await service.handleNavigationRequest(navigationRequest);

      verify(
        navigatorState.pushNamedAndRemoveUntil(
          navigationRequest.destination,
          any,
          arguments: anyNamed('arguments'),
        ),
      );
    });

    test('Verify IssuanceNavigationRequest triggers navigation', () async {
      // Allow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);

      final navigationRequest = NavigationRequest.issuance(
        argument: const IssuanceScreenArgument(isQrCode: false, uri: '/mock'),
      );
      await service.handleNavigationRequest(navigationRequest);

      verify(
        navigatorState.pushNamedAndRemoveUntil(
          navigationRequest.destination,
          any,
          arguments: anyNamed('arguments'),
        ),
      );
    });

    test('Verify SignNavigationRequest triggers navigation', () async {
      // Allow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);

      final navigationRequest = NavigationRequest.sign(argument: const SignScreenArgument(uri: '/mock'));
      await service.handleNavigationRequest(navigationRequest);

      verify(
        navigatorState.pushNamedAndRemoveUntil(
          navigationRequest.destination,
          any,
          arguments: anyNamed('arguments'),
        ),
      );
    });

    test('Verify preNavigationActions are executed before the navigation occurs', () async {
      // Allow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);

      await service.handleNavigationRequest(const GenericNavigationRequest('/mock'));

      verifyInOrder([mockPerformPreNavigationActionsUseCase.invoke(any), navigatorKey.currentState]);
    });

    test('When navigation prerequisites are not fulfilled, do not trigger actual navigation', () async {
      // Disallow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => false);

      await service.handleNavigationRequest(const GenericNavigationRequest('/mock'));

      // Make sure navigation was NOT triggered
      verifyNever(navigatorKey.currentState);
    });
  });

  group('processQueue', () {
    test('Verify request is not queued if not explicitly requested', () async {
      // Disallow navigation, meaning the request should be queued if that is requested
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => false);
      await service.handleNavigationRequest(const GenericNavigationRequest('/mock'));

      // Now allow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);
      // And process any queue if it exists
      await service.processQueue();

      // Make sure navigation was NOT triggered
      verifyNever(navigatorKey.currentState);
    });

    test('Verify request is queued when requested', () async {
      // Disallow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => false);
      await service.handleNavigationRequest(const GenericNavigationRequest('/mock'), queueIfNotReady: true);

      // Now allow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);
      // And process any queue if it exists
      await service.processQueue();

      // Make sure navigation was triggered
      verify(navigatorKey.currentState);
    });

    test(
      'Verify navigation does not occur when prerequisites are not met',
      () async {
        // Disallow navigation
        when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => false);

        // Queue event
        await service.handleNavigationRequest(const GenericNavigationRequest('/mock'), queueIfNotReady: true);

        // Attempt to process queue
        await service.processQueue();

        // Verify queue was not processed
        verifyZeroInteractions(navigatorKey.currentState);
      },
    );
  });

  group('dialog', () {
    test('Verify no dialog is pushed when context is not mounted', () async {
      when(context.mounted).thenAnswer((_) => false);
      await service.showDialog(WalletDialogType.idleWarning);
      verifyNever(navigatorState.push(any));
    });
  });
}
