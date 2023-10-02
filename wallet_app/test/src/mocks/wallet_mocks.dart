import 'package:get_it/get_it.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/pid/pid_repository.dart';
import 'package:wallet/src/data/repository/wallet/wallet_repository.dart';
import 'package:wallet/src/data/service/app_lifecycle_service.dart';
import 'package:wallet/src/domain/usecase/deeplink/decode_deeplink_usecase.dart';
import 'package:wallet/src/domain/usecase/network/check_has_internet_usecase.dart';
import 'package:wallet/src/domain/usecase/pid/accept_offered_pid_usecase.dart';
import 'package:wallet/src/domain/usecase/pid/update_pid_issuance_status_usecase.dart';
import 'package:wallet/src/domain/usecase/pin/check_pin_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/is_wallet_initialized_with_pid_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/observe_wallet_lock_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/setup_mocked_wallet_usecase.dart';
import 'package:wallet/src/util/extension/bloc_extension.dart';
import 'package:wallet/src/util/mapper/locale_mapper.dart';
import 'package:wallet/src/wallet_core/typed/typed_wallet_core.dart';
import 'package:wallet/src/wallet_core/wallet_core.dart';

import 'wallet_mocks.mocks.dart';

export 'wallet_mocks.mocks.dart';

/// Mock mappers
@GenerateNiceMocks([MockSpec<LocaleMapper>()])

/// Mock repositories
@GenerateNiceMocks([MockSpec<PidRepository>()])
@GenerateNiceMocks([MockSpec<WalletRepository>()])

/// Mock services
@GenerateNiceMocks([MockSpec<TypedWalletCore>()])

/// Mock use cases
@GenerateNiceMocks([MockSpec<DecodeDeeplinkUseCase>()])
@GenerateNiceMocks([MockSpec<UpdatePidIssuanceStatusUseCase>()])
@GenerateNiceMocks([MockSpec<IsWalletInitializedWithPidUseCase>()])
@GenerateNiceMocks([MockSpec<ObserveWalletLockUseCase>()])
@GenerateNiceMocks([MockSpec<CheckPinUseCase>()])
@GenerateNiceMocks([MockSpec<SetupMockedWalletUseCase>()])
@GenerateNiceMocks([MockSpec<CheckHasInternetUseCase>()])
@GenerateNiceMocks([MockSpec<AcceptOfferedPidUseCase>()])

/// Core
@GenerateNiceMocks([MockSpec<WalletCore>()])

/// Constants
const kMockPidIssuanceUrl = 'https://example.org';

/// Class that provides the generated mocks with a very
/// basic or no stubbing. Stubs can be overwritten or the mocks
/// can always be instantiated directly. The main intention here
/// allow us to instantiate classes under tests in a simple way,
/// i.e. `xxRepository(Mocks.create(), Mocks.create(), Mocks.create())`
/// When you need more control over what a mock returns you should
/// most likely instantiate the mock directly in your test class.
class Mocks {
  Mocks._();

  static final sl = GetIt.asNewInstance();
  static var isInitialized = false;

  static void initialize() {
    // Core
    sl.registerFactory<WalletCore>(() => MockWalletCore());

    // Services
    sl.registerFactory<AppLifecycleService>(() => AppLifecycleService());
    sl.registerFactory<TypedWalletCore>(() => getTypedWalletCoreMock());

    // Use cases
    sl.registerFactory<DecodeDeeplinkUseCase>(() => MockDecodeDeeplinkUseCase());
    sl.registerFactory<UpdatePidIssuanceStatusUseCase>(() => MockUpdatePidIssuanceStatusUseCase());
    sl.registerFactory<IsWalletInitializedWithPidUseCase>(() => MockIsWalletInitializedWithPidUseCase());
    sl.registerFactory<ObserveWalletLockUseCase>(() => MockObserveWalletLockUseCase());
    sl.registerFactory<CheckPinUseCase>(() => MockCheckPinUseCase());
    sl.registerFactory<SetupMockedWalletUseCase>(() => MockSetupMockedWalletUseCase());
    sl.registerFactory<CheckHasInternetUseCase>(() {
      final mock = MockCheckHasInternetUseCase();
      when(mock.invoke()).thenAnswer((realInvocation) async => true);
      BlocExtensions.checkHasInternetUseCase = mock;
      return mock;
    });
    sl.registerFactory<AcceptOfferedPidUseCase>(() => MockAcceptOfferedPidUseCase());

    // Repositories
    sl.registerFactory<PidRepository>(() => getMockPidRepository());
    sl.registerFactory<WalletRepository>(() => MockWalletRepository());

    // Mappers
    sl.registerFactory<LocaleMapper>(() => MockLocaleMapper());

    isInitialized = true;
  }

  static MockTypedWalletCore getTypedWalletCoreMock() {
    final mock = MockTypedWalletCore();
    when(mock.createPidIssuanceRedirectUri()).thenAnswer((_) async => kMockPidIssuanceUrl);
    return mock;
  }

  static MockPidRepository getMockPidRepository() {
    final mock = MockPidRepository();
    when(mock.getPidIssuanceUrl()).thenAnswer((_) async => kMockPidIssuanceUrl);
    return mock;
  }

  static T create<T extends Object>() {
    if (!isInitialized) initialize();
    return sl.get<T>();
  }
}
