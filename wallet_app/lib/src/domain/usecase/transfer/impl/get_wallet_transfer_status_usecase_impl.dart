import 'dart:math';

import '../../../model/transfer/wallet_transfer_status.dart';
import '../get_wallet_transfer_status_usecase.dart';

class GetWalletTransferStatusUseCaseImpl extends GetWalletTransferStatusUseCase {
  @override
  Stream<WalletTransferStatus> invoke() async* {
    // TODO(Rob): Mock implementation, implement once core supports get_wallet_transfer_status
    yield WalletTransferStatus.transferring;
    await Future.delayed(const Duration(seconds: 5));
    if (Random.secure().nextBool()) {
      yield WalletTransferStatus.error;
    } else {
      yield WalletTransferStatus.success;
    }
  }
}
