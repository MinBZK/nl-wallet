import '../../model/configuration/maintenance_state.dart';
import '../wallet_usecase.dart';

abstract class ObserveMaintenanceStateUseCase extends WalletUseCase {
  Stream<MaintenanceState> invoke();
}
