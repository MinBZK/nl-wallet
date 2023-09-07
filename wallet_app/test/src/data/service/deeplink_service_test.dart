import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/bridge_generated.dart';
import 'package:wallet/src/data/service/deeplink_service.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';
import 'package:wallet/src/wallet_core/typed_wallet_core.dart';

import '../../mocks/wallet_mocks.dart';

void main() {
  late DeeplinkService service;
  late MockTypedWalletCore mockWalletCore;
  late MockDecodeDeeplinkUseCase mockDecodeDeeplinkUseCase;
  late MockIsWalletInitializedWithPidUseCase isWalletInitializedWithPidUseCase;
  late MockUpdatePidIssuanceStatusUseCase updatePidIssuanceStatusUseCase;
  late MockNavigatorKey navigatorKey;

  setUp(() {
    /// Mock the uni_links package
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(const MethodChannel('uni_links/messages'), (MethodCall methodCall) async {
      if (methodCall.method == 'getInitialLink') return null;
      return null;
    });

    mockWalletCore = Mocks.create<TypedWalletCore>() as MockTypedWalletCore;
    mockDecodeDeeplinkUseCase = MockDecodeDeeplinkUseCase();
    isWalletInitializedWithPidUseCase = MockIsWalletInitializedWithPidUseCase();
    updatePidIssuanceStatusUseCase = MockUpdatePidIssuanceStatusUseCase();
    navigatorKey = MockNavigatorKey();

    service = DeeplinkService(
      navigatorKey,
      mockDecodeDeeplinkUseCase,
      updatePidIssuanceStatusUseCase,
      isWalletInitializedWithPidUseCase,
      Mocks.create(),
      Mocks.create(),
      mockWalletCore,
      Mocks.create(),
    );
  });

  group('processUri', () {
    test('Wallet core should not be called when the DecodeDeeplinkUsecase can resolve the url', () async {
      when(mockDecodeDeeplinkUseCase.invoke(any)).thenReturn(NavigationRequest('/mock'));
      service.processUri(Uri.parse('https://example.org'));
      verifyZeroInteractions(mockWalletCore);
    });

    test('A navigation event should be triggered when a supported deeplink url is provided', () async {
      // Allow deeplink_service to navigate
      when(isWalletInitializedWithPidUseCase.invoke()).thenAnswer((_) async => true);

      // Navigation is triggered when supported deeplink is provided
      when(mockDecodeDeeplinkUseCase.invoke(any)).thenReturn(NavigationRequest('/mock'));
      await service.processUri(Uri.parse('https://example.org'));

      // Make sure navigation was triggered (note: currently only shallow validation by checking interaction with the navigator)
      verify(navigatorKey.currentState);
    });

    test(
      'Wallet core should be called when the DecodeDeeplinkUsecase can not resolve the url',
      () async {
        service.processUri(Uri.parse('https://example.org'));
        verify(mockWalletCore.processUri(any));
      },
    );

    test(
      'Result should be passed on to the updateDigidAuthStatusUseCase when the result is relevant',
      () async {
        when(mockWalletCore.processUri(any)).thenAnswer(
          (_) => Stream.value(ProcessUriEvent.pidIssuance(event: PidIssuanceEvent.success(previewCards: List.empty()))),
        );
        await service.processUri(Uri.parse('https://example.org'));
        verify(updatePidIssuanceStatusUseCase.invoke(any));
      },
    );

    test(
      'Result should not be passed on to the updateDigidAuthStatusUseCase when the result is irrelevant',
      () async {
        when(mockWalletCore.processUri(any)).thenAnswer((_) => Stream.error('Error'));
        await service.processUri(Uri.parse('https://example.org'));
        verifyNever(updatePidIssuanceStatusUseCase.invoke(any));
      },
    );
  });

  group('processQueue', () {
    test('No navigation requested when queue is empty', () async {
      when(mockDecodeDeeplinkUseCase.invoke(any)).thenReturn(NavigationRequest('/mock'));
      service.processUri(Uri.parse('https://example.org'));
      verifyZeroInteractions(mockWalletCore);
    });

    test('Navigation requested when queue is filled but navigation can now be done', () async {
      // Provide NavigationRequest
      when(mockDecodeDeeplinkUseCase.invoke(any)).thenReturn(NavigationRequest('/mock'));
      // Make sure it gets queued
      await service.processUri(Uri.parse('https://example.org'));
      // Allow queue to be processed
      when(isWalletInitializedWithPidUseCase.invoke()).thenAnswer((_) async => true);
      // Process the queue
      await service.processQueue();
      // Make sure navigation was triggered (note: currently only shallow validation by checking interaction with the navigator)
      verify(navigatorKey.currentState);
    });
  });
}

// ignore: must_be_immutable
class MockNavigatorKey extends Mock implements GlobalKey<NavigatorState> {}
