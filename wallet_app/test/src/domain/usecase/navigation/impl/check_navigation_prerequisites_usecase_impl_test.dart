import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';
import 'package:wallet/src/domain/usecase/navigation/impl/check_navigation_prerequisites_usecase_impl.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late MockWalletRepository mockWalletRepository;

  late CheckNavigationPrerequisitesUseCaseImpl usecase;

  setUp(() {
    mockWalletRepository = MockWalletRepository();
    usecase = CheckNavigationPrerequisitesUseCaseImpl(mockWalletRepository);
  });

  group('invoke', () {
    test('Conditions are NOT met when walletUnlocked is required and wallet is currently locked', () async {
      when(mockWalletRepository.isLockedStream).thenAnswer((_) => Stream.value(true));
      final result = await usecase.invoke([NavigationPrerequisite.walletUnlocked]);
      expect(result, isFalse);
    });

    test('Conditions are met when walletUnlocked is required and wallet is currently unlocked', () async {
      when(mockWalletRepository.isLockedStream).thenAnswer((_) => Stream.value(false));
      final result = await usecase.invoke([NavigationPrerequisite.walletUnlocked]);
      expect(result, isTrue);
    });

    test('Conditions are NOT met when walletInitialized is required and wallet is currently not initialized', () async {
      when(mockWalletRepository.isRegistered()).thenAnswer((_) async => false);
      final result = await usecase.invoke([NavigationPrerequisite.walletInitialized]);
      expect(result, isFalse);
    });

    test('Conditions are met when walletInitialized is required and wallet is currently initialized', () async {
      when(mockWalletRepository.isRegistered()).thenAnswer((_) async => true);
      final result = await usecase.invoke([NavigationPrerequisite.walletInitialized]);
      expect(result, isTrue);
    });

    test('Conditions are NOT met when pidInitialized is required and wallet does not contain pid', () async {
      when(mockWalletRepository.containsPid()).thenAnswer((_) async => false);
      final result = await usecase.invoke([NavigationPrerequisite.pidInitialized]);
      expect(result, isFalse);
    });

    test('Conditions are met when pidInitialized is required and wallet contains pid', () async {
      when(mockWalletRepository.containsPid()).thenAnswer((_) async => true);
      final result = await usecase.invoke([NavigationPrerequisite.pidInitialized]);
      expect(result, isTrue);
    });

    test('When no prerequisites are specified, the result should be true', () async {
      when(mockWalletRepository.containsPid()).thenAnswer((_) async => true);
      final result = await usecase.invoke([]);
      expect(result, isTrue);
    });
  });
}
