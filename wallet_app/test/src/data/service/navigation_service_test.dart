import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/service/navigation_service.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';

import '../../mocks/wallet_mocks.dart';

void main() {
  late NavigationService service;
  late MockCheckNavigationPrerequisitesUseCase mockCheckNavigationPrerequisitesUseCase;
  late MockPerformPreNavigationActionsUseCase mockPerformPreNavigationActionsUseCase;
  late MockNavigatorKey navigatorKey;
  late MockNavigatorState navigatorState;

  setUp(() {
    provideDummy<NavigationRequest>(const GenericNavigationRequest('/mock_destination'));
    navigatorKey = MockNavigatorKey();
    navigatorState = MockNavigatorState();
    when(navigatorKey.currentState).thenReturn(navigatorState);
    // Usecases
    mockCheckNavigationPrerequisitesUseCase = MockCheckNavigationPrerequisitesUseCase();
    mockPerformPreNavigationActionsUseCase = MockPerformPreNavigationActionsUseCase();

    service = NavigationService(
      navigatorKey,
      mockCheckNavigationPrerequisitesUseCase,
      mockPerformPreNavigationActionsUseCase,
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

      final navigationRequest = PidIssuanceNavigationRequest('/mock');
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

      final navigationRequest = DisclosureNavigationRequest('/mock');
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

      final navigationRequest = IssuanceNavigationRequest('/mock');
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

      final navigationRequest = SignNavigationRequest('/mock');
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

    test('Verify assertion error is thrown when trying to process the queue while prerequisites are still unmet',
        () async {
      // Disallow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => false);
      await service.handleNavigationRequest(const GenericNavigationRequest('/mock'), queueIfNotReady: true);

      // And process any queue if it exists, while app is still NOT ready
      expect(() async => service.processQueue(), throwsAssertionError);
    });
  });
}

// ignore: must_be_immutable
class MockNavigatorKey extends Mock implements GlobalKey<NavigatorState> {}
