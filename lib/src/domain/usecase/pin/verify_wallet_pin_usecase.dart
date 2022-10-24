import '../../../wallet_constants.dart';

class VerifyWalletPinUseCase {
  VerifyWalletPinUseCase();

  Future<bool> verify(String pin) async {
    await Future.delayed(kDefaultMockDelay);
    return pin == kMockPin;
  }
}
