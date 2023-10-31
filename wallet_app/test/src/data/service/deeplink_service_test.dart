import 'dart:async';

import 'package:app_links/app_links.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/service/app_lifecycle_service.dart';
import 'package:wallet/src/data/service/deeplink_service.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';

import '../../mocks/wallet_mocks.dart';

void main() {
  late MockAppLinks appLinks;
  late DeeplinkService service;
  late MockDecodeUriUseCase mockDecodeUriUseCase;
  late MockCheckNavigationPrerequisitesUseCase mockCheckNavigationPrerequisitesUseCase;
  late MockPerformPreNavigationActionsUseCase mockPerformPreNavigationActionsUseCase;
  late MockNavigatorKey navigatorKey;
  late AppLifecycleService mockAppLifecycleService;

  setUp(() {
    appLinks = MockAppLinks();
    navigatorKey = MockNavigatorKey();
    mockAppLifecycleService = AppLifecycleService();
    // Usecases
    mockCheckNavigationPrerequisitesUseCase = MockCheckNavigationPrerequisitesUseCase();
    mockPerformPreNavigationActionsUseCase = MockPerformPreNavigationActionsUseCase();
    mockDecodeUriUseCase = MockDecodeUriUseCase();

    service = DeeplinkService(
      appLinks,
      navigatorKey,
      mockDecodeUriUseCase,
      mockCheckNavigationPrerequisitesUseCase,
      mockPerformPreNavigationActionsUseCase,
      mockAppLifecycleService,
    );
  });

  group('processUri', () {
    test('A navigation event should be triggered when a supported deeplink url is provided', () async {
      // Allow deeplink_service to navigate
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);

      // Navigation is triggered when supported deeplink is provided
      when(mockDecodeUriUseCase.invoke(any)).thenAnswer((_) async => GenericNavigationRequest('/mock'));
      await appLinks.mockUriEvent(Uri.parse('https://example.org'));

      // Make sure navigation was triggered (note: currently only shallow validation by checking interaction with the navigator)
      verify(navigatorKey.currentState);
    });
  });

  group('processQueue', () {
    test('Navigation requested when queue is filled but navigation can now be done', () async {
      // Provide NavigationRequest
      when(mockDecodeUriUseCase.invoke(any)).thenAnswer((_) async => GenericNavigationRequest('/mock'));
      // Make sure it gets queued
      await appLinks.mockUriEvent(Uri.parse('https://example.org'));
      // Allow queue to be processed
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);
      // Process the queue
      await service.processQueue();
      // Make sure navigation was triggered (note: currently only shallow validation by checking interaction with the navigator)
      verify(navigatorKey.currentState);
    });
  });
}

// ignore: must_be_immutable
class MockNavigatorKey extends Mock implements GlobalKey<NavigatorState> {}

class MockAppLinks implements AppLinks {
  MockAppLinks();

  final StreamController<Uri> _uriStreamController = StreamController();

  Future<void> mockUriEvent(Uri uri) async {
    _uriStreamController.add(uri);
    // Give the event some time to propagate
    await Future.delayed(kResumeDebounceDuration * 1.5);
  }

  @override
  Stream<String> get allStringLinkStream => throw UnimplementedError();

  @override
  Stream<Uri> get allUriLinkStream => _uriStreamController.stream;

  @override
  Future<Uri?> getInitialAppLink() async => null;

  @override
  Future<String?> getInitialAppLinkString() => throw UnimplementedError();

  @override
  Future<Uri?> getLatestAppLink() => throw UnimplementedError();

  @override
  Future<String?> getLatestAppLinkString() => throw UnimplementedError();

  @override
  Stream<String> get stringLinkStream => throw UnimplementedError();

  @override
  Stream<Uri> get uriLinkStream => throw UnimplementedError();
}
