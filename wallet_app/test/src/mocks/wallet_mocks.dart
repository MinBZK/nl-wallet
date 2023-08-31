import 'package:get_it/get_it.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/authentication/digid_auth_repository.dart';
import 'package:wallet/src/data/repository/wallet/wallet_repository.dart';
import 'package:wallet/src/data/service/app_lifecycle_service.dart';
import 'package:wallet/src/domain/usecase/auth/update_digid_auth_status_usecase.dart';
import 'package:wallet/src/domain/usecase/deeplink/decode_deeplink_usecase.dart';
import 'package:wallet/src/domain/usecase/pin/check_pin_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/is_wallet_initialized_with_pid_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/observe_wallet_lock_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/setup_mocked_wallet_usecase.dart';
import 'package:wallet/src/wallet_core/typed_wallet_core.dart';

import 'wallet_mocks.mocks.dart';

export 'wallet_mocks.mocks.dart';

/// Mock repositories
@GenerateNiceMocks([MockSpec<DigidAuthRepository>()])
@GenerateNiceMocks([MockSpec<WalletRepository>()])

/// Mock services
@GenerateNiceMocks([MockSpec<TypedWalletCore>()])

///Mock usecases
@GenerateNiceMocks([MockSpec<DecodeDeeplinkUseCase>()])
@GenerateNiceMocks([MockSpec<UpdateDigidAuthStatusUseCase>()])
@GenerateNiceMocks([MockSpec<IsWalletInitializedWithPidUseCase>()])
@GenerateNiceMocks([MockSpec<ObserveWalletLockUseCase>()])
@GenerateNiceMocks([MockSpec<CheckPinUseCase>()])
@GenerateNiceMocks([MockSpec<SetupMockedWalletUseCase>()])

/// Constants
const kMockDigidAuthUrl = 'https://example.org';

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
    // Services
    sl.registerFactory<AppLifecycleService>(() => AppLifecycleService());
    sl.registerFactory<TypedWalletCore>(() => getTypedWalletCoreMock());
    // Usecases
    sl.registerFactory<DecodeDeeplinkUseCase>(() => MockDecodeDeeplinkUseCase());
    sl.registerFactory<UpdateDigidAuthStatusUseCase>(() => MockUpdateDigidAuthStatusUseCase());
    sl.registerFactory<IsWalletInitializedWithPidUseCase>(() => MockIsWalletInitializedWithPidUseCase());
    sl.registerFactory<ObserveWalletLockUseCase>(() => MockObserveWalletLockUseCase());
    sl.registerFactory<CheckPinUseCase>(() => MockCheckPinUseCase());
    sl.registerFactory<SetupMockedWalletUseCase>(() => MockSetupMockedWalletUseCase());
    // Repositories
    sl.registerFactory<DigidAuthRepository>(() => getMockDigidAuthRepository());
    sl.registerFactory<WalletRepository>(() => MockWalletRepository());

    isInitialized = true;
  }

  static MockTypedWalletCore getTypedWalletCoreMock() {
    final mock = MockTypedWalletCore();
    when(mock.createPidIssuanceRedirectUri()).thenAnswer((_) async => kMockDigidAuthUrl);
    return mock;
  }

  static MockDigidAuthRepository getMockDigidAuthRepository() {
    final mock = MockDigidAuthRepository();
    when(mock.getAuthUrl()).thenAnswer((_) async => kMockDigidAuthUrl);
    return mock;
  }

  static T create<T extends Object>() {
    if (!isInitialized) initialize();
    return sl.get<T>();
  }
}
