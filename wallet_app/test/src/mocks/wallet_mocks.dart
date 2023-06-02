import 'package:get_it/get_it.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/authentication/digid_auth_repository.dart';
import 'package:wallet/src/data/service/app_lifecycle_service.dart';
import 'package:wallet/src/domain/usecase/auth/update_digid_auth_status_usecase.dart';
import 'package:wallet/src/domain/usecase/deeplink/decode_deeplink_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/is_wallet_initialized_with_pid_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/observe_wallet_lock_usecase.dart';
import 'package:wallet/src/wallet_core/typed_wallet_core.dart';

import 'wallet_mocks.mocks.dart';

export 'wallet_mocks.mocks.dart';

/// Mock repositories
@GenerateNiceMocks([MockSpec<DigidAuthRepository>()])

/// Mock services
@GenerateNiceMocks([MockSpec<TypedWalletCore>()])
@GenerateNiceMocks([MockSpec<AppLifecycleService>()])

///Mock usecases
@GenerateNiceMocks([MockSpec<DecodeDeeplinkUseCase>()])
@GenerateNiceMocks([MockSpec<UpdateDigidAuthStatusUseCase>()])
@GenerateNiceMocks([MockSpec<IsWalletInitializedWithPidUseCase>()])
@GenerateNiceMocks([MockSpec<ObserveWalletLockUseCase>()])

/// Constants
const kMockDigidAuthUrl = 'https://example.org';

/// Class that provides the generated mocks with a very
/// basic stubbing. Stubs can be overwritten or the mocks
/// can always be instantiated directly.
class Mocks {
  Mocks._();

  static final sl = GetIt.asNewInstance();
  static var isInitialized = false;

  static void initialize() {
    sl.registerFactory<MockAppLifecycleService>(() => MockAppLifecycleService());
    sl.registerFactory<MockDecodeDeeplinkUseCase>(() => MockDecodeDeeplinkUseCase());
    sl.registerFactory<MockUpdateDigidAuthStatusUseCase>(() => MockUpdateDigidAuthStatusUseCase());
    sl.registerFactory<MockIsWalletInitializedWithPidUseCase>(() => MockIsWalletInitializedWithPidUseCase());
    sl.registerFactory<MockObserveWalletLockUseCase>(() => MockObserveWalletLockUseCase());
    sl.registerFactory<MockTypedWalletCore>(() => getTypedWalletCoreMock());
    sl.registerFactory<MockDigidAuthRepository>(() => getMockDigidAuthRepository());
    isInitialized = true;
  }

  static MockTypedWalletCore getTypedWalletCoreMock() {
    final mock = MockTypedWalletCore();
    when(mock.getDigidAuthUrl()).thenAnswer((_) async => kMockDigidAuthUrl);
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
